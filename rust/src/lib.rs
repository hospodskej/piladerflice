//! Pila Derflice: the sawmill's website and shop, in Rust.
//!
//! The app is built from:
//! - [`request`]: language from the URL (`/at` = German) and request info
//! - [`i18n`]: translations from `config/locales/*.yml`
//! - [`assets`]: fingerprinted static files
//! - [`views`]: templates and the shared page layout
//! - [`routes`]: one module per area of the site

pub mod assets;
pub mod i18n;
pub mod locale;
pub mod request;
pub mod routes;
pub mod views;

use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use tower::ServiceBuilder;
use tower::util::MapRequestLayer;

use crate::assets::Assets;
use crate::i18n::Translations;
use crate::views::Templates;

pub struct Config {
    /// The repository root: translations, images, CSS and JavaScript are
    /// shared with the Rails app and read from there.
    pub app_root: PathBuf,
    /// This crate's folder (templates, vendored JavaScript).
    pub crate_root: PathBuf,
    /// Development: templates and assets are re-read when they change.
    pub development: bool,
}

impl Config {
    /// Paths relative to this source tree; good for development and tests.
    pub fn from_source_tree(development: bool) -> Self {
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let app_root = crate_root
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| crate_root.clone());
        Self {
            app_root,
            crate_root,
            development,
        }
    }
}

pub struct Inner {
    pub config: Config,
    pub translations: Arc<Translations>,
    pub assets: Arc<Assets>,
    pub templates: Templates,
}

/// Shared by every request; cheap to clone.
#[derive(Clone)]
pub struct AppState(Arc<Inner>);

impl Deref for AppState {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.0
    }
}

impl AppState {
    pub fn new(config: Config) -> Result<Self> {
        let translations = Arc::new(Translations::load(&config.app_root.join("config/locales"))?);
        let assets = Arc::new(Assets::new(
            assets::default_sources(&config.app_root, &config.crate_root),
            config.development,
        )?);
        let templates = Templates::new(
            config.crate_root.join("templates"),
            translations.clone(),
            assets.clone(),
            config.development,
        );
        Ok(Self(Arc::new(Inner {
            config,
            translations,
            assets,
            templates,
        })))
    }
}

/// The whole site as one service: the /at prefix is taken off before routing.
pub fn app(
    state: AppState,
) -> impl tower::Service<
    axum::extract::Request,
    Response = axum::response::Response,
    Error = std::convert::Infallible,
    Future = impl Send,
> + Clone
+ Send
+ 'static {
    let router: Router = routes::router(state);
    ServiceBuilder::new()
        .layer(MapRequestLayer::new(request::strip_locale_prefix))
        .service(router)
}
