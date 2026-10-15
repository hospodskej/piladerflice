//! HTML pages. Templates live in `templates/` and use Jinja syntax
//! (MiniJinja):
//!
//! - `{{ t("nav.domu") }}`: a translation (`t("key", km=50)` fills `{km}`)
//! - `{{ path("/kontakt#cenik") }}`: a site link in the current language
//! - `{{ asset("logo-wm.webp") }}`: the fingerprinted URL of a static file
//!
//! - [`templates`]: loading templates and the functions above
//! - [`page`]: per-page title, description, breadcrumbs, status
//! - [`layout`]: rendering a page inside the site layout
//! - [`breadcrumbs`], [`seo`]: what the layout derives from the URL
//! - [`cart_view`]: the cart as the pages show it (header widget, cart page, checkout)
//! - [`turbo`], [`fragment`]: Turbo Stream answers to forms; [`redirect`]: after a form
//! - [`error_pages`]: the static 422 and 500 pages
//! - [`text`]: small text helpers (escaping, squish, truncate)
//! - [`form_helpers`]: form fields for the admin templates
//! - [`admin_layout`]: rendering an admin page inside the admin layout

pub mod admin_layout;
pub mod breadcrumbs;
pub mod cart_view;
pub mod error_pages;
pub mod form_helpers;
pub mod fragment;
mod functions;
pub mod layout;
pub mod page;
pub mod redirect;
pub mod seo;
pub mod templates;
pub mod text;
pub mod turbo;

pub use breadcrumbs::Crumb;
pub use layout::render;
pub use page::Page;
pub use templates::Templates;
pub use text::escape;
