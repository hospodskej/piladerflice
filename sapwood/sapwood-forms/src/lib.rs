//! HTML forms for Axum.
//!
//! Every extractor here checks the request first: the `Origin` header must
//! be the site itself, and the form must carry the session's CSRF token (the
//! field [`CSRF_FIELD`], or the `X-CSRF-Token` header). It needs
//! [sapwood-session](https://docs.rs/sapwood-session)'s `SessionLayer` on the
//! router. A handler never sees a forged form; the request gets a 422 instead.
//!
//! - [`FormData`]: the fields as they came, for forms with repeated fields
//!   or a few values to read by hand;
//! - [`Form<T>`]: the fields as your own `#[derive(Deserialize)]` struct;
//! - [`UploadForm`]: a form with files (`multipart/form-data`).
//!
//! ```no_run
//! use axum::response::Redirect;
//! use sapwood_forms::{Errors, Form};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Signup {
//!     email: String,
//!     #[serde(default, deserialize_with = "sapwood_forms::de::checkbox")]
//!     newsletter: bool,
//! }
//!
//! async fn signup(Form(form): Form<Signup>) -> Result<Redirect, String> {
//!     let mut errors = Errors::new();
//!     if !form.email.contains('@') {
//!         errors.add("email", "Enter your e-mail address");
//!     }
//!     if !errors.is_empty() {
//!         return Err(errors.messages().join(", "));
//!     }
//!     Ok(Redirect::to("/thanks"))
//! }
//! ```
//!
//! In your templates, put the token into every form:
//! `<input type="hidden" name="csrf_token" value="{{ csrf_token }}">`.

pub mod de;
mod errors;
mod extract;
mod form_data;
mod upload;

pub use errors::Errors;
pub use extract::{Form, Rejected};
pub use form_data::FormData;
pub use upload::{UploadForm, UploadedFile};

/// The form field that carries the CSRF token.
pub const CSRF_FIELD: &str = sapwood_session::csrf::FIELD;

/// Forms without files may be this big at most; bigger ones are refused.
pub const MAX_FORM_BYTES: usize = 64 * 1024;
