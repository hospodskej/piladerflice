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
//!
//! A login lives in a second cookie that lasts (like Rails' permanent
//! `session_id` cookie): it holds the id of the `sessions` row, so closing
//! the browser doesn't sign the customer out.

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
    /// Where to go after signing in (the page that asked for it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_to: Option<String>,
}

struct Inner {
    data: SessionData,
    changed: bool,
    /// The `sessions` row id of the login, if signed in.
    login: Option<i64>,
    login_changed: bool,
}

/// A handle to this request's session. Cloning it is cheap; all clones see
/// the same data. Changes are saved into the cookie when the response goes
/// out (see [`layer`]).
#[derive(Clone)]
pub struct Session(Arc<Mutex<Inner>>);

impl Session {
    pub fn new(data: SessionData) -> Self {
        Self::with_login(data, None)
    }

    pub fn with_login(data: SessionData, login: Option<i64>) -> Self {
        Session(Arc::new(Mutex::new(Inner {
            data,
            changed: false,
            login,
            login_changed: false,
        })))
    }

    /// The `sessions` row of the current login.
    pub fn login(&self) -> Option<i64> {
        self.0.lock().expect("session lock").login
    }

    /// Signs in (`Some(row id)`) or out (`None`); saved with the response.
    pub fn set_login(&self, login: Option<i64>) {
        let mut inner = self.0.lock().expect("session lock");
        inner.login = login;
        inner.login_changed = true;
    }

    /// The new login to save, when it changed: Some(Some(id)) or Some(None).
    fn changed_login(&self) -> Option<Option<i64>> {
        let inner = self.0.lock().expect("session lock");
        inner.login_changed.then_some(inner.login)
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
