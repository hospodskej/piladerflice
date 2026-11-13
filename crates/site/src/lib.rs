//! The Pila Derflice website. Pages are rendered on the server; the
//! interactive parts are islands that hydrate in the browser as WASM.

#![recursion_limit = "512"]

pub mod app;
pub mod assets;
pub mod autocomplete;
pub mod cart_totals;
pub mod cart_islands;
pub mod cart_view;
pub mod catalog;
pub mod i18n;
pub mod inquiry_form;
pub mod islands;
pub mod layout;
pub mod pages;
pub mod seo;
pub mod server_fns;
pub mod shop;
pub mod time_fmt;
pub mod variant_filter;
pub mod visitor;

#[cfg(feature = "ssr")]
pub mod cart_ops;
#[cfg(feature = "ssr")]
pub mod forms;
#[cfg(feature = "ssr")]
pub mod limits;
#[cfg(feature = "ssr")]
pub mod mail;
#[cfg(feature = "ssr")]
pub mod orders;
#[cfg(feature = "ssr")]
pub mod plain;
#[cfg(feature = "ssr")]
pub mod routing;
#[cfg(feature = "ssr")]
pub mod session;
#[cfg(feature = "ssr")]
pub mod state;
