//! The visitor's session (sapwood-session): what we remember between their
//! requests, in an encrypted cookie. Ours holds the cart, the checkout
//! details, the order just placed and where to go after signing in; the
//! framework adds the form protection token and the one-time message.
//!
//! A login lives in a second cookie that lasts 20 years: it holds the id of
//! the `sessions` row, so closing the browser doesn't sign the customer out.

use std::time::Duration;

use serde::{Deserialize, Serialize};

pub use sapwood_session::{Flash, Key, csrf};

use crate::shop::cart::Cart;
use crate::shop::checkout::CheckoutState;

pub type Session = sapwood_session::Session<SessionData>;

/// Everything we keep in the session cookie.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct SessionData {
    #[serde(default)]
    pub cart: Cart,
    #[serde(default, skip_serializing_if = "CheckoutState::is_empty")]
    pub checkout: CheckoutState,
    /// The order just placed, for the thank-you page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_order_id: Option<i64>,
    /// Where to go after signing in (the page that asked for it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_to: Option<String>,
}

const LOGIN_LIFETIME: Duration = Duration::from_secs(20 * 365 * 24 * 60 * 60);

/// The middleware that gives every request its [`Session`].
pub fn layer(key: Key, secure: bool) -> sapwood_session::SessionLayer<SessionData> {
    sapwood_session::SessionLayer::with_key(key)
        .cookie_name("_pila_session")
        .secure(secure)
        .login_cookie("_pila_login", LOGIN_LIFETIME)
}

/// The `sessions` row of the current login.
pub fn login_id(session: &Session) -> Option<i64> {
    session.login().and_then(|id| id.parse().ok())
}
