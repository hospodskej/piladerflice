//! Files that are compiled into the server.

/// The address of the bundled stylesheet; changes whenever its content does.
pub const STYLESHEET_HREF: &str = include_str!(concat!(env!("OUT_DIR"), "/site_css_href.txt"));

/// All the site's styles in one file (see `build.rs`).
#[cfg(feature = "ssr")]
pub const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/site.css"));
