//! What the pages are built from besides their components (`ui/`):
//!
//! - [`page`]: per-page title, description, breadcrumbs, status
//! - [`breadcrumbs`], [`seo`]: what the layout derives from the URL
//! - [`cart_view`]: the cart as the pages show it (header widget, cart page, checkout)
//! - [`redirect`]: after a form
//! - [`error_pages`]: the static 422 and 500 pages
//! - [`text`]: small text helpers (escaping, squish, truncate)

pub mod breadcrumbs;
pub mod cart_view;
pub mod error_pages;
pub mod page;
pub mod redirect;
pub mod seo;
pub mod text;

pub use breadcrumbs::Crumb;
pub use page::Page;
pub use text::escape;
