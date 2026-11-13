//! The Pila Derflice website. Pages are rendered on the server; the
//! interactive parts are islands that hydrate in the browser as WASM.

#![recursion_limit = "512"]

pub mod app;
pub mod assets;
pub mod cart_totals;
pub mod catalog;
pub mod i18n;
pub mod islands;
pub mod layout;
pub mod pages;
pub mod seo;
pub mod server_fns;
pub mod shop;
pub mod visitor;

#[cfg(feature = "ssr")]
pub mod session;
#[cfg(feature = "ssr")]
pub mod state;
