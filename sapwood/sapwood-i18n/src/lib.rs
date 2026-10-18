//! Translations for server-rendered sites.
//!
//! One YAML file per language (`locales/cs.yml`, `locales/de.yml`...), looked
//! up by dotted key (`"nav.home"`), with `{name}` placeholders and an optional
//! fallback language for keys a translation doesn't have yet.
//!
//! ```yaml
//! # locales/en.yml
//! nav:
//!   home: Home
//! cart:
//!   added: "{product} is in your cart"
//! ```
//!
//! ```
//! use sapwood_i18n::Translations;
//!
//! let translations = Translations::from_sources([
//!     ("en", "cart:\n  added: \"{product} is in your cart\"\nnav:\n  home: Home\n"),
//!     ("de", "nav:\n  home: Startseite\n"),
//! ])
//! .unwrap()
//! .with_fallback("en");
//!
//! assert_eq!(translations.text("de", "nav.home", &[]), "Startseite");
//! // Not translated yet: the fallback language fills in.
//! assert_eq!(translations.text("de", "cart.added", &[("product", "Oak")]), "Oak is in your cart");
//! ```
//!
//! With the `minijinja` feature, [`jinja::function`] gives MiniJinja
//! templates a `t("key", name=value)` function. (Components, e.g. Leptos,
//! call [`Translations::text`] themselves.)

mod error;
mod interpolate;
#[cfg(feature = "minijinja")]
pub mod jinja;
mod translations;

pub use error::Error;
pub use interpolate::{escape_html, interpolate};
pub use serde_json::Value;
pub use translations::{Translations, missing};
