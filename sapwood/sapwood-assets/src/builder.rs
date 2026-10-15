//! Setting up an [`Assets`] index.

use std::path::PathBuf;

use crate::Assets;

/// Where the files come from and how they are served; see [`Assets::builder`].
#[derive(Debug, Clone)]
pub struct Builder {
    pub(crate) sources: Vec<(PathBuf, String)>,
    pub(crate) url_prefix: String,
    pub(crate) watch: bool,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            url_prefix: "/assets".to_string(),
            watch: false,
        }
    }
}

impl Builder {
    /// Serves the files under `dir`, named by their path inside it
    /// (`dir/icons/cart.svg` is `"icons/cart.svg"`). When two folders have a
    /// file of the same name, the folder added first wins.
    pub fn dir(self, dir: impl Into<PathBuf>) -> Self {
        self.dir_as(dir, "")
    }

    /// Like [`dir`](Self::dir), with `prefix` in front of the names:
    /// `.dir_as("vendor", "vendor/")` makes `vendor/x.js` `"vendor/x.js"`.
    pub fn dir_as(mut self, dir: impl Into<PathBuf>, prefix: impl Into<String>) -> Self {
        self.sources.push((dir.into(), prefix.into()));
        self
    }

    /// The URL path the files are served under; `/assets` by default.
    pub fn url_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.url_prefix = prefix.into().trim_end_matches('/').to_string();
        self
    }

    /// Whether [`Assets::refresh`] looks for new and edited files (for
    /// development). Off by default: the files are indexed once.
    pub fn watch(mut self, watch: bool) -> Self {
        self.watch = watch;
        self
    }

    /// Reads and fingerprints every file. Folders that don't exist are
    /// skipped.
    pub fn build(self) -> std::io::Result<Assets> {
        Assets::from_builder(self)
    }
}
