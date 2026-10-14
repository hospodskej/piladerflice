//! Settings: where the files, the database and the uploads are.

use std::path::PathBuf;

pub struct Config {
    /// The repository root: translations, images, CSS and JavaScript are
    /// shared with the Rails app and read from there.
    pub app_root: PathBuf,
    /// This crate's folder (templates, vendored JavaScript).
    pub crate_root: PathBuf,
    /// Development: templates and assets are re-read when they change.
    pub development: bool,
    /// e.g. `sqlite:../storage/development.sqlite3`
    pub database_url: String,
    /// Where uploaded images live (Active Storage's Disk service root).
    pub storage_root: PathBuf,
}

impl Config {
    /// Paths relative to this source tree; good for development and tests.
    pub fn from_source_tree(development: bool) -> Self {
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let app_root = crate_root
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| crate_root.clone());
        let database_url = format!("sqlite:{}", app_root.join("storage/development.sqlite3").display());
        let storage_root = app_root.join("storage");
        Self {
            app_root,
            crate_root,
            development,
            database_url,
            storage_root,
        }
    }
}
