//! The index of files and their fingerprinted URLs.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::SystemTime;

use crate::Builder;
use crate::fingerprint::{fingerprinted_path, walk};

#[derive(Clone)]
struct Asset {
    file: PathBuf,
    fingerprinted: String,
    modified: Option<SystemTime>,
    len: u64,
}

#[derive(Default)]
struct Index {
    /// name ("css/site.css") -> asset
    by_name: BTreeMap<String, Asset>,
    /// fingerprinted name ("css/site-1a2b3c4d.css") -> name
    by_fingerprint: BTreeMap<String, String>,
}

/// The static files and their fingerprinted URLs. Build one with
/// [`Assets::builder`] and share it (`Arc<Assets>`).
pub struct Assets {
    sources: Vec<(PathBuf, String)>,
    url_prefix: String,
    watch: bool,
    index: RwLock<Index>,
}

/// A requested file: where it is, and whether it was asked for by its
/// fingerprinted URL (safe to cache forever).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub file: PathBuf,
    pub fingerprinted: bool,
}

impl Assets {
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub(crate) fn from_builder(builder: Builder) -> std::io::Result<Self> {
        let assets = Self {
            sources: builder.sources,
            url_prefix: builder.url_prefix,
            watch: builder.watch,
            index: RwLock::new(Index::default()),
        };
        assets.scan()?;
        Ok(assets)
    }

    /// Looks for new and edited files when the index watches (development);
    /// does nothing otherwise. Cheap: only changed files are hashed again.
    /// Call it once per page view.
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

        let old = self.index.read().expect("asset index lock");
        let mut index = Index::default();
        for (name, file) in found {
            let meta = std::fs::metadata(&file)?;
            let modified = meta.modified().ok();
            let asset = match old.by_name.get(&name) {
                Some(known) if known.file == file && known.modified == modified && known.len == meta.len() => {
                    known.clone()
                }
                _ => Asset {
                    fingerprinted: fingerprinted_path(&name, &std::fs::read(&file)?),
                    file,
                    modified,
                    len: meta.len(),
                },
            };
            index.by_fingerprint.insert(asset.fingerprinted.clone(), name.clone());
            index.by_name.entry(name).or_insert(asset);
        }
        drop(old);
        *self.index.write().expect("asset index lock") = index;
        Ok(())
    }

    /// The URL of a file: `"/assets/logo-1a2b3c4d.webp"` for `"logo.webp"`.
    /// An unknown name gets its plain URL and a warning in the log, so a typo
    /// shows up as a broken image rather than an error page.
    pub fn url(&self, name: &str) -> String {
        let name = name.trim_start_matches('/');
        match self.index.read().expect("asset index lock").by_name.get(name) {
            Some(asset) => format!("{}/{}", self.url_prefix, asset.fingerprinted),
            None => {
                tracing::warn!("unknown asset {name}");
                format!("{}/{name}", self.url_prefix)
            }
        }
    }

    pub fn exists(&self, name: &str) -> bool {
        self.index.read().expect("asset index lock").by_name.contains_key(name)
    }

    /// Every file name, sorted.
    pub fn names(&self) -> Vec<String> {
        self.index
            .read()
            .expect("asset index lock")
            .by_name
            .keys()
            .cloned()
            .collect()
    }

    /// The URLs of every file with this extension, sorted by name: e.g. all
    /// stylesheets (`"css"`) to link in the page head.
    pub fn urls_with_extension(&self, extension: &str) -> Vec<String> {
        let suffix = format!(".{extension}");
        self.index
            .read()
            .expect("asset index lock")
            .by_name
            .iter()
            .filter(|(name, _)| name.ends_with(&suffix))
            .map(|(_, asset)| format!("{}/{}", self.url_prefix, asset.fingerprinted))
            .collect()
    }

    /// The URL path the files are served under (`"/assets"`).
    pub fn url_prefix(&self) -> &str {
        &self.url_prefix
    }

    /// The file for a path under the URL prefix: a fingerprinted name
    /// (`"logo-1a2b3c4d.webp"`) or a plain one (`"logo.webp"`).
    pub fn find(&self, requested: &str) -> Option<Found> {
        let index = self.index.read().expect("asset index lock");
        if let Some(name) = index.by_fingerprint.get(requested) {
            return index.by_name.get(name).map(|asset| Found {
                file: asset.file.clone(),
                fingerprinted: true,
            });
        }
        index.by_name.get(requested).map(|asset| Found {
            file: asset.file.clone(),
            fingerprinted: false,
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {

    use super::*;

    /// A folder of test files, removed when dropped.
    pub struct Dir(pub PathBuf);

    impl Dir {
        pub fn new(files: &[(&str, &str)]) -> Self {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static N: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "sapwood-assets-{}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::Relaxed)
            ));
            for (name, content) in files {
                let file = dir.join(name);
                std::fs::create_dir_all(file.parent().unwrap()).unwrap();
                std::fs::write(file, content).unwrap();
            }
            Dir(dir)
        }

        pub fn path(&self, sub: &str) -> PathBuf {
            self.0.join(sub)
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn names_files_by_their_path_and_content() {
        let dir = Dir::new(&[("css/site.css", "body{}"), ("logo.svg", "<svg/>"), (".gitkeep", "")]);
        let assets = Assets::builder().dir(&dir.0).build().unwrap();
        assert_eq!(assets.names(), ["css/site.css", "logo.svg"]);
        let url = assets.url("css/site.css");
        assert!(url.starts_with("/assets/css/site-") && url.ends_with(".css"), "{url}");
        assert_eq!(assets.url("/logo.svg"), assets.url("logo.svg"));
        assert_eq!(assets.url("missing.png"), "/assets/missing.png");
        assert_eq!(assets.urls_with_extension("css"), std::slice::from_ref(&url));

        let found = assets.find(url.trim_start_matches("/assets/")).unwrap();
        assert!(found.fingerprinted);
        assert_eq!(found.file, dir.path("css/site.css"));
        assert!(!assets.find("css/site.css").unwrap().fingerprinted);
        assert!(assets.find("css/site-00000000.css").is_none());
    }

    #[test]
    fn the_first_folder_wins_and_prefixes_apply() {
        let a = Dir::new(&[("app.js", "a")]);
        let b = Dir::new(&[("app.js", "b"), ("lib.js", "l")]);
        let assets = Assets::builder()
            .dir(&a.0)
            .dir(&b.0)
            .dir_as(&b.0, "vendor/")
            .url_prefix("/static/")
            .build()
            .unwrap();
        assert_eq!(assets.find("app.js").unwrap().file, a.path("app.js"));
        assert!(assets.exists("vendor/lib.js"));
        assert!(assets.url("lib.js").starts_with("/static/lib-"));
    }

    #[test]
    fn watching_picks_up_edits() {
        let dir = Dir::new(&[("a.css", "one")]);
        let watching = Assets::builder().dir(&dir.0).watch(true).build().unwrap();
        let fixed = Assets::builder().dir(&dir.0).build().unwrap();
        let before = watching.url("a.css");
        std::fs::write(dir.path("a.css"), "two, longer").unwrap();
        std::fs::write(dir.path("b.css"), "new").unwrap();
        watching.refresh();
        fixed.refresh();
        assert_ne!(watching.url("a.css"), before);
        assert!(watching.exists("b.css"));
        assert_eq!(fixed.url("a.css"), before);
        assert!(!fixed.exists("b.css"));
    }

    #[test]
    fn missing_folders_are_skipped() {
        let assets = Assets::builder().dir("/nonexistent/assets").build().unwrap();
        assert!(assets.names().is_empty());
    }
}
