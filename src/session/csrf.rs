//! Protection against cross-site request forgery: another website must not
//! be able to make a visitor's browser submit our forms (put things in their
//! cart, place an order...).
//!
//! Every session gets a random token; our forms send it back as
//! `authenticity_token` (or Turbo sends it as the `X-CSRF-Token` header), and
//! a form without the right token is refused. A request that says it comes
//! from another site (`Origin`) is refused too.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use subtle::ConstantTimeEq;

/// A new random token: 32 bytes, base64.
pub fn new_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Whether `given` is the session's token. Compared in constant time, so the
/// time it takes says nothing about how much of it was right.
pub fn matches(expected: &str, given: &str) -> bool {
    !expected.is_empty() && expected.as_bytes().ct_eq(given.as_bytes()).into()
}

/// Whether the browser's `Origin` header (when it sends one) is our own site.
pub fn origin_allowed(origin: Option<&str>, base_url: &str) -> bool {
    match origin {
        None | Some("null") => origin.is_none(),
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
        assert!(origin_allowed(Some("https://pila.cz"), "https://pila.cz"));
        assert!(!origin_allowed(Some("https://evil.example"), "https://pila.cz"));
        assert!(!origin_allowed(Some("null"), "https://pila.cz"));
    }
}
