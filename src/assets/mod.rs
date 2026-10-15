//! Static files (images, CSS, JavaScript), served the way Propshaft does it in
//! Rails: every file gets a URL with a fingerprint of its content
//! (`/assets/logo-wm-1a2b3c4d.webp`), so browsers may cache it forever and a
//! changed file simply gets a new URL. The files are in `assets/` (images,
//! CSS), `javascript/` and, for Turbo and Stimulus, `vendor/javascript/`.
//!
//! In development the index notices edited files on the next page view.
//!
//! - this file: the index of files and their fingerprinted URLs
//! - [`fingerprint`]: finding the files and fingerprinting them
//! - [`importmap`]: the JavaScript import map
//! - [`serve`]: answering /assets/... requests

mod fingerprint;
mod importmap;
mod serve;

pub use serve::serve;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::SystemTime;

use fingerprint::{digested_path, walk};

/// Where assets come from: (directory, prefix of their logical path).
pub fn default_sources(root: &Path) -> Vec<(PathBuf, String)> {
    vec![
        (root.join("assets/images"), String::new()),
        (root.join("assets/stylesheets"), String::new()),
        (root.join("javascript"), String::new()),
        (root.join("vendor/javascript"), String::new()),
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
                logical.ends_with(".css") && asset.file.to_string_lossy().contains("assets/stylesheets/")
            })
            .map(|(_, asset)| format!("/assets/{}", asset.digested))
            .collect()
    }

    /// The file for a requested path, and whether it was the fingerprinted
    /// URL (cache forever) or the plain one.
    pub(super) fn find(&self, requested: &str) -> Option<(PathBuf, bool)> {
        let index = self.index.read().unwrap();
        if let Some(logical) = index.by_digested.get(requested) {
            return index.by_logical.get(logical).map(|asset| (asset.file.clone(), true));
        }
        index.by_logical.get(requested).map(|asset| (asset.file.clone(), false))
    }
}
