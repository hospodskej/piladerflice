//! The visitor's session: what we remember between their requests (the
//! cart, checkout details, the form protection token, a one-time message).
//!
//! It lives in an encrypted cookie, like Rails' cookie store: the visitor
//! can't read or change it, and the server keeps nothing.
//!
//! - this file: the data, and the [`Session`] handle handlers use
//! - [`layer`]: loads the session before a request, saves it after
//! - [`cookie_store`]: encrypting it into the cookie and back
//! - [`csrf`]: the token that proves a form came from our own page
//! - [`flash`]: a message shown once on the next page

pub mod cookie_store;
pub mod csrf;
pub mod flash;
pub mod layer;

use std::sync::{Arc, Mutex};

use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};

pub use flash::Flash;

use crate::shop::cart::Cart;
use crate::shop::checkout::CheckoutState;

/// Everything stored in the session cookie.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct SessionData {
    #[serde(default)]
    pub cart: Cart,
    #[serde(default, skip_serializing_if = "CheckoutState::is_empty")]
    pub checkout: CheckoutState,
    #[serde(default)]
    pub csrf: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flash: Option<Flash>,
    /// The order just placed, for the thank-you page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_order_id: Option<i64>,
}

struct Inner {
    data: SessionData,
    changed: bool,
}

/// A handle to this request's session. Cloning it is cheap; all clones see
/// the same data. Changes are saved into the cookie when the response goes
/// out (see [`layer`]).
#[derive(Clone)]
pub struct Session(Arc<Mutex<Inner>>);

impl Session {
    pub fn new(data: SessionData) -> Self {
        Session(Arc::new(Mutex::new(Inner { data, changed: false })))
    }

    /// Looks at the session without changing it.
    pub fn read<R>(&self, f: impl FnOnce(&SessionData) -> R) -> R {
        f(&self.0.lock().expect("session lock").data)
    }

    /// Changes the session; it will be saved with the response.
    pub fn update<R>(&self, f: impl FnOnce(&mut SessionData) -> R) -> R {
        let mut inner = self.0.lock().expect("session lock");
        inner.changed = true;
        f(&mut inner.data)
    }

    /// The message set by the previous request, removed so it shows once.
    pub fn take_flash(&self) -> Option<Flash> {
        if self.read(|data| data.flash.is_none()) {
            return None;
        }
        self.update(|data| data.flash.take())
    }

    pub fn set_flash(&self, flash: Flash) {
        self.update(|data| data.flash = Some(flash));
    }

    /// The form protection token, created the first time a page needs it
    /// (not in the layer: the parallel image requests of a first visit would
    /// each create a different one).
    pub fn csrf_token(&self) -> String {
        let token = self.read(|data| data.csrf.clone());
        if !token.is_empty() {
            return token;
        }
        self.update(|data| {
            data.csrf = csrf::new_token();
            data.csrf.clone()
        })
    }

    fn changed(&self) -> Option<SessionData> {
        let inner = self.0.lock().expect("session lock");
        inner.changed.then(|| inner.data.clone())
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Session(..)")
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Session {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Session>()
            .cloned()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "session layer missing"))
    }
}
