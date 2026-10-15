//! Pila Derflice: the sawmill's website and shop, in Rust.
//!
//! The app is built from:
//! - [`request`]: language from the URL (`/at` = German) and request info
//! - [`i18n`]: translations from `locales/*.yml`
//! - [`assets`]: fingerprinted static files
//! - [`views`]: templates and the shared page layout
//! - [`db`] and [`models`]: the SQLite database and its tables
//! - [`money`]: crowns, and euros for the Austrian site
//! - [`routes`]: one module per area of the site

pub mod accounts;
pub mod admin;
pub mod assets;
pub mod commands;
pub mod config;
pub mod czech_time;
pub mod db;
pub mod forms;
pub mod i18n;
pub mod locale;
pub mod mail;
pub mod models;
pub mod money;
pub mod request;
pub mod routes;
pub mod session;
pub mod shop;
pub mod state;
pub mod storage;
pub mod views;

pub use config::Config;
pub use state::{AppState, Services};

use axum::Router;
use tower::ServiceBuilder;
use tower::util::MapRequestLayer;

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
    let router: Router =
        routes::router(state.clone()).layer(session::layer(state.cookie_key.clone(), state.config.secure_cookies));
    ServiceBuilder::new()
        .layer(MapRequestLayer::new(request::strip_locale_prefix))
        .service(router)
}
