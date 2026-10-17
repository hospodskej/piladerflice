//! The site's HTML: Leptos components, rendered on the server by the
//! handlers in `routes/`. The interactive parts are islands (`islands/`),
//! which the browser brings to life with WebAssembly.
//!
//! - [`context`]: what components read while rendering: `t("key")`,
//!   `path("/kontakt")`, `asset("logo.webp")`, the form token
//! - [`render`]: turning a page into a response
//! - [`layout`]: the page around the content (head, header, footer...)
//! - [`icons`]: SVG pictures used in several places
//! - one module per area of the site, like `routes/`

pub mod account;
pub mod auth;
pub mod cart;
pub mod checkout;
pub mod contact;
pub mod context;
pub mod eshop;
pub mod home;
pub mod icons;
pub mod layout;
pub mod legal;
pub mod not_found;
pub mod render;
pub mod services;
pub mod shared;
pub mod sortiment;

pub use context::{
    CsrfField, Ui, asset, csrf_token, current_path, locale, path, t, t_html, t_html_with, t_list, t_with,
};
