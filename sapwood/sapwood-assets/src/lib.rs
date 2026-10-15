//! Fingerprinted static files: images, CSS and JavaScript served at URLs
//! that contain a hash of their content (`/assets/logo-1a2b3c4d.webp`), so
//! browsers may cache them forever and a changed file simply gets a new URL.
//!
//! ```no_run
//! use std::sync::Arc;
//! use sapwood_assets::{Assets, Importmap};
//!
//! let assets = Arc::new(
//!     Assets::builder()
//!         .dir("assets")             // assets/logo.webp is "logo.webp"
//!         .dir("javascript")
//!         .watch(cfg!(debug_assertions)) // notice edited files in development
//!         .build()?,
//! );
//! assert!(assets.url("logo.webp").starts_with("/assets/logo-"));
//!
//! let importmap = Importmap::new().pin("application", "application.js").pin_all_from("controllers");
//! let json = importmap.json(&assets); // for <script type="importmap">
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! With the `axum` feature (on by default), [`Assets::router`] serves the
//! files: fingerprinted URLs with `Cache-Control: immutable`, plain paths
//! (`/assets/logo.webp`) too, but uncached.

mod builder;
mod fingerprint;
mod importmap;
mod index;
#[cfg(feature = "axum")]
mod serve;

pub use builder::Builder;
pub use fingerprint::fingerprinted_path;
pub use importmap::Importmap;
pub use index::{Assets, Found};
