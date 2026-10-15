//! Encrypted cookie sessions for Axum.
//!
//! What a visitor's session holds (a cart, a half-filled form, where to go
//! after signing in) is your own type. It travels in a cookie encrypted and
//! authenticated with AES-256-GCM, so the visitor can't read or change it and
//! the server stores nothing. The framework adds three things every site
//! needs:
//!
//! - a **CSRF token** proving a form came from your own page
//!   ([`Session::csrf_token`], [`Session::verify_csrf`]);
//! - a **flash message** shown once on the next page ([`Session::set_flash`]);
//! - an optional **login cookie** that outlives the browser session
//!   ([`Session::set_login`]), holding e.g. the id of a login record.
//!
//! ```no_run
//! use axum::{Router, routing::get};
//! use sapwood_session::{Flash, Session, SessionLayer};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Clone, Default, Serialize, Deserialize)]
//! struct Visit {
//!     pages_seen: u32,
//! }
//!
//! async fn page(session: Session<Visit>) -> String {
//!     let seen = session.update(|visit| {
//!         visit.pages_seen += 1;
//!         visit.pages_seen
//!     });
//!     format!("page {seen} of your visit")
//! }
//!
//! let secret = std::env::var("SECRET_KEY_BASE").unwrap(); // at least 64 random bytes
//! let app: Router = Router::new()
//!     .route("/", get(page))
//!     .layer(SessionLayer::<Visit>::new(secret.as_bytes()).cookie_name("_shop_session"));
//! ```
//!
//! Requests that don't touch the session (images, CSS) set no cookie.
//! Changing the secret signs everyone out and empties their sessions.
//! Browsers drop cookies over 4 kB, so keep the data small: ids, not records.

pub mod csrf;
mod flash;
mod layer;
mod session;
mod store;

pub use cookie::Key;
pub use flash::Flash;
pub use layer::{SessionLayer, SessionService};
pub use session::Session;

/// What your session type must be: (de)serializable, with an empty default.
/// Any `#[derive(Clone, Default, Serialize, Deserialize)]` struct is.
pub trait SessionData:
    serde::Serialize + serde::de::DeserializeOwned + Default + Clone + Send + Sync + 'static
{
}

impl<T> SessionData for T where
    T: serde::Serialize + serde::de::DeserializeOwned + Default + Clone + Send + Sync + 'static
{
}
