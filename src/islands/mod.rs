//! The interactive parts of the pages ("islands"). They are rendered on the
//! server with the rest of the page, and compiled to WebAssembly too: in
//! the browser, each one comes alive in place. Everything an island shows
//! comes in its props (texts included: the translations stay on the
//! server); what it changes goes back through server functions.
//!
//! - [`shared`]: state several islands share (the cart, the open menu...)
//! - [`page`]: the page around the content (theme, menu, cookie banner...)
//! - [`cart`]: the header widget, add-to-cart, the cart page
//! - [`checkout`]: the terms box, the purchase report
//! - [`quote`]: the quote request form's rows and lists
//! - [`widgets`]: accordions, carousels, the fitting headline
//! - [`account`]: the phone prefix and profile picture pickers
//! - [`browser`]: helpers for the browser's page

pub mod account;
mod browser;
pub mod cart;
pub mod checkout;
pub mod page;
pub mod quote;
#[cfg(feature = "ssr")]
mod server;
pub mod shared;
pub mod widgets;
