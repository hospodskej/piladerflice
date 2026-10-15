//! Turning the session into an encrypted cookie and back. The `cookie`
//! crate's private jar encrypts and authenticates the value (AES-256-GCM)
//! with a key derived from `SECRET_KEY_BASE`.

use cookie::{Cookie, CookieJar, Key, SameSite};

use super::SessionData;

pub const COOKIE_NAME: &str = "_pila_session";

/// Browsers drop cookies over 4 kB; warn well before that.
const SIZE_WARNING: usize = 3800;

/// The session from the request's Cookie header; empty when there is none
/// or it can't be decrypted (an old or tampered cookie).
pub fn load(cookie_header: Option<&str>, key: &Key) -> SessionData {
    read(cookie_header, key, COOKIE_NAME)
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

/// The decrypted value of one of our cookies.
pub fn read(cookie_header: Option<&str>, key: &Key, name: &str) -> Option<String> {
    let mut jar = CookieJar::new();
    for cookie in cookie_header
        .into_iter()
        .flat_map(Cookie::split_parse_encoded)
        .flatten()
    {
        if cookie.name() == name {
            jar.add_original(cookie.into_owned());
        }
    }
    jar.private(key).get(name).map(|cookie| cookie.value().to_string())
}

/// The Set-Cookie header value for the session.
pub fn store(data: &SessionData, key: &Key, secure: bool) -> String {
    let json = serde_json::to_string(data).unwrap_or_default();
    let header = write(COOKIE_NAME, json, key, secure, None);
    if header.len() > SIZE_WARNING {
        tracing::warn!(
            "session cookie is {} bytes; browsers drop cookies over 4096",
            header.len()
        );
    }
    header
}

/// A Set-Cookie header value for an encrypted cookie. `max_age` None means
/// "until the browser closes".
pub fn write(name: &str, value: String, key: &Key, secure: bool, max_age: Option<cookie::time::Duration>) -> String {
    let mut jar = CookieJar::new();
    let mut cookie = Cookie::new(name.to_string(), value);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_secure(secure);
    if let Some(max_age) = max_age {
        cookie.set_max_age(max_age);
    }
    jar.private_mut(key).add(cookie);
    jar.get(name)
        .map(|cookie| cookie.encoded().to_string())
        .unwrap_or_default()
}

/// A Set-Cookie header value that deletes a cookie.
pub fn delete(name: &str, secure: bool) -> String {
    let mut cookie = Cookie::new(name.to_string(), "");
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_secure(secure);
    cookie.make_removal();
    cookie.encoded().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_rejects_tampering() {
        let key = Key::derive_from(&[7u8; 64]);
        let data = SessionData {
            csrf: "abc".into(),
            last_order_id: Some(5),
            ..SessionData::default()
        };
        let header = store(&data, &key, false);
        let value = header.split(';').next().unwrap();
        assert_eq!(load(Some(value), &key), data);
        assert!(!value.contains("abc"), "the cookie must be encrypted");

        let tampered = value.replacen('=', "=x", 1);
        assert_eq!(load(Some(&tampered), &key), SessionData::default());
        assert_eq!(load(Some(value), &Key::derive_from(&[8u8; 64])), SessionData::default());
        assert_eq!(load(None, &key), SessionData::default());
    }
}
