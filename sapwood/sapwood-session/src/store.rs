//! Turning the session into encrypted cookies and back.

use cookie::time::Duration;
use cookie::{Cookie, CookieJar, Key, SameSite};
use serde::{Deserialize, Serialize};

use crate::Flash;

/// Browsers drop cookies over 4 kB; warn well before that.
const SIZE_WARNING: usize = 3800;

/// What the session cookie holds: the app's data plus the framework's.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct Stored<T> {
    #[serde(default)]
    pub data: T,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub csrf: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flash: Option<Flash>,
}

/// How the cookies are written.
pub(crate) struct CookieSettings {
    pub key: Key,
    pub name: String,
    pub secure: bool,
    /// The login cookie's name and lifetime, when logins are used.
    pub login: Option<(String, Duration)>,
}

/// The decrypted value of the cookie `name`, if the request has a valid one.
pub(crate) fn read(cookie_header: Option<&str>, key: &Key, name: &str) -> Option<String> {
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

/// The session in the request: empty when there is none, or when it can't
/// be decrypted or read (tampered with, or written with another secret).
pub(crate) fn load<T: serde::de::DeserializeOwned + Default>(
    cookie_header: Option<&str>,
    settings: &CookieSettings,
) -> Stored<T> {
    read(cookie_header, &settings.key, &settings.name)
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

/// The Set-Cookie value for the session.
pub(crate) fn store<T: Serialize>(stored: &Stored<T>, settings: &CookieSettings) -> String {
    let json = serde_json::to_string(stored).unwrap_or_default();
    let header = write(&settings.name, json, settings, None);
    if header.len() > SIZE_WARNING {
        tracing::warn!(
            "session cookie is {} bytes; browsers drop cookies over 4096",
            header.len()
        );
    }
    header
}

/// The Set-Cookie value of an encrypted cookie; `max_age` None lasts until
/// the browser closes.
pub(crate) fn write(name: &str, value: String, settings: &CookieSettings, max_age: Option<Duration>) -> String {
    let mut jar = CookieJar::new();
    let mut cookie = base_cookie(name, value, settings.secure);
    if let Some(max_age) = max_age {
        cookie.set_max_age(max_age);
    }
    jar.private_mut(&settings.key).add(cookie);
    jar.get(name)
        .map(|cookie| cookie.encoded().to_string())
        .unwrap_or_default()
}

/// The Set-Cookie value that deletes a cookie.
pub(crate) fn delete(name: &str, secure: bool) -> String {
    let mut cookie = base_cookie(name, String::new(), secure);
    cookie.make_removal();
    cookie.encoded().to_string()
}

fn base_cookie(name: &str, value: String, secure: bool) -> Cookie<'static> {
    let mut cookie = Cookie::new(name.to_string(), value);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_secure(secure);
    cookie
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(secret: u8) -> CookieSettings {
        CookieSettings {
            key: Key::derive_from(&[secret; 64]),
            name: "_s".into(),
            secure: true,
            login: None,
        }
    }

    #[test]
    fn round_trips_and_rejects_tampering() {
        let stored = Stored {
            data: vec![1, 2, 3],
            csrf: "abc".into(),
            flash: Some(Flash::Notice("hi".into())),
        };
        let header = store(&stored, &settings(7));
        assert!(header.contains("Secure") && header.contains("HttpOnly") && header.contains("SameSite=Lax"));
        let value = header.split(';').next().unwrap();
        assert_eq!(load::<Vec<i32>>(Some(value), &settings(7)), stored);
        assert!(!value.contains("abc"), "the cookie must be encrypted");

        let tampered = value.replacen('=', "=x", 1);
        assert_eq!(load::<Vec<i32>>(Some(&tampered), &settings(7)), Stored::default());
        assert_eq!(load::<Vec<i32>>(Some(value), &settings(8)), Stored::default());
        assert_eq!(load::<Vec<i32>>(None, &settings(7)), Stored::default());
    }

    #[test]
    fn deleting_expires_the_cookie() {
        let header = delete("_login", false);
        assert!(header.starts_with("_login=;"), "{header}");
        assert!(header.contains("Max-Age=0"));
    }
}
