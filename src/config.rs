//! Settings: where the files, the database and the uploads are, the secret
//! for the session cookie, e-mail and the delivery distance service.

use std::path::PathBuf;

use crate::mail::MailSettings;

/// Only for development and tests. Production must set `SECRET_KEY_BASE`.
const DEVELOPMENT_SECRET: &str = "development-only-secret-key-for-the-session-cookie-of-pila-derflice-0123456789";

pub struct Config {
    /// The project folder (or, on a server, the release folder):
    /// translations, images, CSS and `public/` are read from it.
    pub root: PathBuf,
    /// Development: assets and the WebAssembly are re-read when they change.
    pub development: bool,
    /// e.g. `sqlite:storage/development.sqlite3`
    pub database_url: String,
    /// Where uploaded images live (Active Storage's folder layout).
    pub storage_root: PathBuf,
    /// Encrypts the session cookie; at least 64 characters.
    pub secret_key_base: String,
    /// Send the session cookie over HTTPS only (production).
    pub secure_cookies: bool,
    /// openrouteservice key for the delivery distance; None = no distance.
    pub ors_api_key: Option<String>,
    pub mail: MailSettings,
    /// The islands' WebAssembly build (`cargo leptos build` writes it to
    /// target/site/pkg; a release has it in pkg/).
    pub pkg_dir: PathBuf,
}

impl Config {
    /// The icons and the static error pages.
    pub fn public_dir(&self) -> PathBuf {
        self.root.join("public")
    }

    /// Paths in this source tree; good for development and tests. Nothing is
    /// read from the environment here (see `from_env`).
    pub fn from_source_tree(development: bool) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let database_url = format!("sqlite:{}", root.join("storage/development.sqlite3").display());
        let storage_root = root.join("storage/uploads");
        let pkg_dir = root.join("target/site/pkg");
        Self {
            root,
            development,
            database_url,
            storage_root,
            secret_key_base: DEVELOPMENT_SECRET.into(),
            secure_cookies: false,
            ors_api_key: None,
            mail: MailSettings::outbox_only(),
            pkg_dir,
        }
    }

    /// The settings of a running server: the source tree, overridden by
    /// environment variables. Production refuses to start without a secret.
    pub fn from_env(development: bool) -> anyhow::Result<Self> {
        let var = |name: &str| std::env::var(name).ok().filter(|value| !value.is_empty());
        let mut config = Self::from_source_tree(development);
        if let Some(root) = var("APP_ROOT") {
            config.root = PathBuf::from(&root);
            config.database_url = format!("sqlite:{}", config.root.join("storage/production.sqlite3").display());
            config.storage_root = config.root.join("storage/uploads");
            config.pkg_dir = config.root.join("pkg");
        }
        if let Some(url) = var("DATABASE_URL") {
            config.database_url = url;
        }
        if let Some(dir) = var("STORAGE_DIR") {
            config.storage_root = dir.into();
        }
        match var("SECRET_KEY_BASE") {
            Some(secret) if secret.len() >= 64 => config.secret_key_base = secret,
            Some(_) => anyhow::bail!("SECRET_KEY_BASE must be at least 64 characters"),
            None if !development => anyhow::bail!("SECRET_KEY_BASE is required in production"),
            None => {}
        }
        config.secure_cookies = !development;
        config.ors_api_key = var("ORS_API_KEY");
        config.mail = MailSettings::from_env();
        Ok(config)
    }
}
