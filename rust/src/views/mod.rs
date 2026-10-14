//! HTML pages. Templates live in `templates/` and use Jinja syntax
//! (MiniJinja), close to the ERB views of the Rails app:
//!
//! - `{{ t("nav.domu") }}`: a translation (`t("key", km=50)` fills `%{km}`)
//! - `{{ path("/kontakt#cenik") }}`: a site link in the current language
//! - `{{ asset("logo-wm.webp") }}`: the fingerprinted URL of a static file
//!
//! - [`templates`]: loading templates and the functions above
//! - [`page`]: per-page title, description, breadcrumbs, status
//! - [`layout`]: rendering a page inside the site layout
//! - [`breadcrumbs`], [`seo`]: what the layout derives from the URL
//! - [`text`]: small text helpers (escaping, Rails' squish and truncate)

pub mod breadcrumbs;
mod functions;
pub mod layout;
pub mod page;
pub mod seo;
pub mod templates;
pub mod text;

pub use breadcrumbs::Crumb;
pub use layout::render;
pub use page::Page;
pub use templates::Templates;
pub use text::escape;
