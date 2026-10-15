//! Protection against cross-site request forgery: another website must not
//! be able to make a visitor's browser submit your forms.
//!
//! Every session gets a random token ([`crate::Session::csrf_token`]); put
//! it in your forms as the field [`FIELD`] (or let JavaScript send it as the
//! [`HEADER`] header) and refuse a form whose token doesn't match
//! ([`crate::Session::verify_csrf`]). Also refuse requests whose `Origin`
//! header names another site ([`origin_allowed`]).

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use subtle::ConstantTimeEq;

/// The form field that carries the token.
pub const FIELD: &str = "csrf_token";
/// The request header that carries the token (Turbo and fetch requests).
pub const HEADER: &str = "x-csrf-token";

/// The session's token as the request brought it (empty when the session
/// has none yet). The [`crate::SessionLayer`] puts it into every request's
/// extensions, so code that doesn't know the session's data type (form
/// extractors) can check a form: `request.extensions().get::<RequestToken>()`.
#[derive(Clone, Debug, Default)]
pub struct RequestToken(pub String);

impl RequestToken {
    /// Whether `given` is the token.
    pub fn verify(&self, given: &str) -> bool {
        matches(&self.0, given)
    }
}

/// A new random token: 32 bytes, base64.
pub fn new_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Whether `given` is the `expected` token. Compared in constant time, so
/// how long it takes says nothing about how much of it was right.
pub fn matches(expected: &str, given: &str) -> bool {
    !expected.is_empty() && expected.as_bytes().ct_eq(given.as_bytes()).into()
}

/// Whether the browser's `Origin` header, when it sends one, is the site
/// itself (`base_url`, e.g. `"https://example.com"`).
pub fn origin_allowed(origin: Option<&str>, base_url: &str) -> bool {
    match origin {
        None => true,
        Some("null") => false,
        Some(origin) => origin.trim_end_matches('/') == base_url.trim_end_matches('/'),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_random_and_compared_exactly() {
        let token = new_token();
        assert_eq!(token.len(), 43);
        assert_ne!(token, new_token());
        assert!(matches(&token, &token));
        assert!(!matches(&token, &token[1..]));
        assert!(!matches("", ""));
    }

    #[test]
    fn only_our_own_origin_is_allowed() {
        assert!(origin_allowed(None, "https://pila.cz"));
        assert!(origin_allowed(Some("https://pila.cz"), "https://pila.cz/"));
        assert!(!origin_allowed(Some("https://evil.example"), "https://pila.cz"));
        assert!(!origin_allowed(Some("null"), "https://pila.cz"));
    }
}
