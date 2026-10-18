//! Pila Derflice: the sawmill's website and shop, in Rust.
//!
//! The app is built from:
//! - [`request`]: language from the URL (`/at` = German) and request info
//! - [`i18n`]: translations from `locales/*.yml`
//! - [`assets`]: fingerprinted static files
//! - [`db`] and [`models`]: the SQLite database and its tables
//! - [`money`]: crowns, and euros for the Austrian site
//! - [`routes`]: one module per area of the site; [`ui`]: the pages as
//!   Leptos components, rendered on the server
//! - [`islands`]: the interactive parts, also compiled to WebAssembly for
//!   the browser (the `hydrate` build)

// Leptos encodes a whole page in its types; big pages nest deeply.
#![recursion_limit = "512"]

#[cfg(feature = "ssr")]
pub mod accounts;
#[cfg(feature = "ssr")]
pub mod admin;
#[cfg(feature = "ssr")]
pub mod assets;
#[cfg(feature = "ssr")]
pub mod commands;
#[cfg(feature = "ssr")]
pub mod config;
#[cfg(feature = "ssr")]
pub mod czech_time;
#[cfg(feature = "ssr")]
pub mod db;
#[cfg(feature = "ssr")]
pub mod forms;
#[cfg(feature = "ssr")]
pub mod i18n;
#[cfg(feature = "ssr")]
pub mod locale;
#[cfg(feature = "ssr")]
pub mod mail;
#[cfg(feature = "ssr")]
pub mod models;
#[cfg(feature = "ssr")]
pub mod money;
#[cfg(feature = "ssr")]
pub mod request;
#[cfg(feature = "ssr")]
pub mod routes;
#[cfg(feature = "ssr")]
pub mod session;
#[cfg(feature = "ssr")]
pub mod shop;
#[cfg(feature = "ssr")]
pub mod state;
#[cfg(feature = "ssr")]
pub mod storage;
#[cfg(feature = "ssr")]
pub mod views;
#[cfg(feature = "ssr")]
pub mod wasm;

pub mod icons;
pub mod islands;
#[cfg(feature = "ssr")]
pub mod ui;

#[cfg(feature = "ssr")]
pub use config::Config;
#[cfg(feature = "ssr")]
pub use state::{AppState, Services};

#[cfg(feature = "ssr")]
use axum::Router;
#[cfg(feature = "ssr")]
use tower::ServiceBuilder;
#[cfg(feature = "ssr")]
use tower::util::MapRequestLayer;

/// The whole site as one service: the /at prefix is taken off before routing.
#[cfg(feature = "ssr")]
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

/// The browser's entry point: the WebAssembly module calls this once
/// loaded, then each island on the page comes alive.
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    islands::page::adopt_stored_theme();
    leptos::mount::hydrate_islands();
}
