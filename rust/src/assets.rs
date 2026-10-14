//! Static files (images, CSS, JavaScript), served the way Propshaft does it in
//! Rails: every file gets a URL with a fingerprint of its content
//! (`/assets/logo-wm-1a2b3c4d.webp`), so browsers may cache it forever and a
//! changed file simply gets a new URL. The files are the Rails app's own
//! (`app/assets`, `app/javascript`) plus Turbo and Stimulus in `vendor/`.
//!
//! In development the index notices edited files on the next page view.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::SystemTime;

use axum::body::Body;
use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::json;
use sha1::{Digest, Sha1};

use crate::AppState;

/// Where assets come from: (directory, prefix of their logical path).
pub fn default_sources(app_root: &Path, crate_root: &Path) -> Vec<(PathBuf, String)> {
    vec![
        (app_root.join("app/assets/images"), String::new()),
        (app_root.join("app/assets/stylesheets"), String::new()),
        (app_root.join("app/javascript"), String::new()),
        (crate_root.join("vendor/javascript"), String::new()),
    ]
}

#[derive(Clone)]
struct Asset {
    file: PathBuf,
    digested: String,
    modified: Option<SystemTime>,
    len: u64,
}

#[derive(Default)]
struct Index {
    /// logical path ("base/global.css") -> asset
    by_logical: BTreeMap<String, Asset>,
    /// digested path ("base/global-1a2b3c4d.css") -> logical path
    by_digested: BTreeMap<String, String>,
}

pub struct Assets {
    sources: Vec<(PathBuf, String)>,
    index: RwLock<Index>,
    watch: bool,
}

impl Assets {
    pub fn new(sources: Vec<(PathBuf, String)>, watch: bool) -> std::io::Result<Self> {
        let assets = Self {
            sources,
            index: RwLock::new(Index::default()),
            watch,
        };
        assets.scan()?;
        Ok(assets)
    }

    /// Re-reads the asset directories, fingerprinting new or changed files
    /// only. Called on each page view in development.
    pub fn refresh(&self) {
        if self.watch
            && let Err(error) = self.scan()
        {
            tracing::warn!("asset refresh failed: {error}");
        }
    }

    fn scan(&self) -> std::io::Result<()> {
        let mut found = Vec::new();
        for (dir, prefix) in &self.sources {
            if dir.is_dir() {
                walk(dir, dir, prefix, &mut found)?;
            }
        }

        let old = self.index.read().unwrap();
        let mut index = Index::default();
        for (logical, file) in found {
            let meta = std::fs::metadata(&file)?;
            let modified = meta.modified().ok();
            let asset = match old.by_logical.get(&logical) {
                Some(known) if known.file == file && known.modified == modified && known.len == meta.len() => {
                    known.clone()
                }
                _ => Asset {
                    digested: digested_path(&logical, &std::fs::read(&file)?),
                    file,
                    modified,
                    len: meta.len(),
                },
            };
            index.by_digested.insert(asset.digested.clone(), logical.clone());
            index.by_logical.entry(logical).or_insert(asset);
        }
        drop(old);
        *self.index.write().unwrap() = index;
        Ok(())
    }

    /// "/assets/logo-wm-1a2b3c4d.webp" for "logo-wm.webp". An unknown file
    /// gets its plain path (and a warning), like a broken image in Rails.
    pub fn url(&self, logical: &str) -> String {
        let logical = logical.trim_start_matches('/');
        match self.index.read().unwrap().by_logical.get(logical) {
            Some(asset) => format!("/assets/{}", asset.digested),
            None => {
                tracing::warn!("unknown asset {logical}");
                format!("/assets/{logical}")
            }
        }
    }

    pub fn exists(&self, logical: &str) -> bool {
        self.index.read().unwrap().by_logical.contains_key(logical)
    }

    /// All stylesheets, in the order Propshaft links them for `:app`.
    pub fn stylesheet_urls(&self) -> Vec<String> {
        let index = self.index.read().unwrap();
        index
            .by_logical
            .iter()
            .filter(|(logical, asset)| {
                logical.ends_with(".css") && asset.file.to_string_lossy().contains("app/assets/stylesheets/")
            })
            .map(|(_, asset)| format!("/assets/{}", asset.digested))
            .collect()
    }

    /// The import map (same pins as `config/importmap.rb`): module name -> URL.
    pub fn importmap(&self) -> Vec<(String, String)> {
        let mut pins = vec![
            ("application".to_string(), self.url("application.js")),
            ("@hotwired/turbo-rails".to_string(), self.url("turbo.min.js")),
            ("@hotwired/stimulus".to_string(), self.url("stimulus.min.js")),
            (
                "@hotwired/stimulus-loading".to_string(),
                self.url("stimulus-loading.js"),
            ),
        ];
        let index = self.index.read().unwrap();
        for folder in ["controllers", "custom"] {
            let prefix = format!("{folder}/");
            for (logical, asset) in index.by_logical.range(prefix.clone()..) {
                if !logical.starts_with(&prefix) {
                    break;
                }
                let Some(module) = logical.strip_suffix(".js") else {
                    continue;
                };
                // controllers/index.js is imported as "controllers".
                let name = module.strip_suffix("/index").unwrap_or(module).to_string();
                pins.push((name, format!("/assets/{}", asset.digested)));
            }
        }
        pins
    }

    pub fn importmap_json(&self) -> String {
        let imports: serde_json::Map<String, serde_json::Value> = self
            .importmap()
            .into_iter()
            .map(|(name, url)| (name, json!(url)))
            .collect();
        serde_json::to_string_pretty(&json!({ "imports": imports })).unwrap_or_default()
    }

    fn find(&self, requested: &str) -> Option<(PathBuf, bool)> {
        let index = self.index.read().unwrap();
        if let Some(logical) = index.by_digested.get(requested) {
            return index.by_logical.get(logical).map(|asset| (asset.file.clone(), true));
        }
        index.by_logical.get(requested).map(|asset| (asset.file.clone(), false))
    }
}

fn walk(root: &Path, dir: &Path, prefix: &str, found: &mut Vec<(String, PathBuf)>) -> std::io::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        if path.is_dir() {
            walk(root, &path, prefix, found)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            found.push((format!("{prefix}{relative}"), path));
        }
    }
    Ok(())
}

/// "base/global.css" + content -> "base/global-1a2b3c4d.css"
fn digested_path(logical: &str, content: &[u8]) -> String {
    let digest = hex::encode(Sha1::digest(content));
    let fingerprint = &digest[..8];
    let (dir, file) = logical
        .rsplit_once('/')
        .map_or(("", logical), |(dir, file)| (dir, file));
    let renamed = match file.rsplit_once('.') {
        Some((stem, ext)) => format!("{stem}-{fingerprint}.{ext}"),
        None => format!("{file}-{fingerprint}"),
    };
    if dir.is_empty() {
        renamed
    } else {
        format!("{dir}/{renamed}")
    }
}

/// GET /assets/*path
pub async fn serve(State(state): State<AppState>, UrlPath(requested): UrlPath<String>) -> Response {
    let Some((file, digested)) = state.assets.find(&requested) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(bytes) = tokio::fs::read(&file).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mime = match file.extension().and_then(|ext| ext.to_str()) {
        Some("js") => "text/javascript".to_string(),
        _ => mime_guess::from_path(&file).first_or_octet_stream().to_string(),
    };
    let cache = if digested {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };

    let mut response = Response::new(Body::from(bytes));
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime).unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(cache));
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprints_keep_the_folder_and_extension() {
        let digested = digested_path("base/global.css", b"body{}");
        assert!(digested.starts_with("base/global-"), "{digested}");
        assert!(digested.ends_with(".css"));
        assert_eq!(digested.len(), "base/global-12345678.css".len());
        assert!(digested_path("turbo.min.js", b"x").starts_with("turbo.min-"));
    }
}
