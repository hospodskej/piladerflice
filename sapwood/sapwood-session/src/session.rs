//! The [`Session`] handle handlers use.

use std::sync::{Arc, Mutex, MutexGuard};

use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;

use crate::store::{self, CookieSettings, Stored};
use crate::{Flash, SessionData, csrf};

struct Inner<T> {
    stored: Stored<T>,
    changed: bool,
    login: Option<String>,
    login_changed: bool,
}

/// This request's session. Take it as a handler argument
/// (`session: Session<MyData>`); cloning is cheap and every clone sees the
/// same data. Changes are saved into the cookie when the response goes out.
pub struct Session<T>(Arc<Mutex<Inner<T>>>);

impl<T> Clone for Session<T> {
    fn clone(&self) -> Self {
        Session(self.0.clone())
    }
}

impl<T: SessionData> Session<T> {
    /// A session with this data and no login, not read from any request
    /// (for tests, or code running outside a request).
    pub fn new(data: T) -> Self {
        Self::from_parts(
            Stored {
                data,
                ..Stored::default()
            },
            None,
        )
    }

    pub(crate) fn from_parts(stored: Stored<T>, login: Option<String>) -> Self {
        Session(Arc::new(Mutex::new(Inner {
            stored,
            changed: false,
            login,
            login_changed: false,
        })))
    }

    fn lock(&self) -> MutexGuard<'_, Inner<T>> {
        self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Looks at the data without changing it.
    pub fn read<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.lock().stored.data)
    }

    /// Changes the data; the new cookie goes out with the response.
    pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut inner = self.lock();
        inner.changed = true;
        f(&mut inner.stored.data)
    }

    /// Shows `flash` on the next page.
    pub fn set_flash(&self, flash: Flash) {
        let mut inner = self.lock();
        inner.changed = true;
        inner.stored.flash = Some(flash);
    }

    /// The message set by the previous request, removed so it shows once.
    pub fn take_flash(&self) -> Option<Flash> {
        let mut inner = self.lock();
        let flash = inner.stored.flash.take();
        inner.changed |= flash.is_some();
        flash
    }

    /// The session's CSRF token, created the first time a page needs it.
    /// (Not up front: the parallel image requests of a first visit would
    /// each create a different one.)
    pub fn csrf_token(&self) -> String {
        let mut inner = self.lock();
        if inner.stored.csrf.is_empty() {
            inner.stored.csrf = csrf::new_token();
            inner.changed = true;
        }
        inner.stored.csrf.clone()
    }

    /// Whether `given` (the form field or header) is this session's token.
    pub fn verify_csrf(&self, given: &str) -> bool {
        csrf::matches(&self.lock().stored.csrf, given)
    }

    /// The login cookie's value (e.g. the id of a login record), if signed in.
    pub fn login(&self) -> Option<String> {
        self.lock().login.clone()
    }

    /// Signs in (`Some(value)`) or out (`None`); saved with the response.
    /// Needs [`crate::SessionLayer::login_cookie`].
    pub fn set_login(&self, login: Option<String>) {
        let mut inner = self.lock();
        inner.login = login;
        inner.login_changed = true;
    }

    /// The Set-Cookie values for what changed during the request.
    pub(crate) fn set_cookie_headers(&self, settings: &CookieSettings) -> Vec<String> {
        let inner = self.lock();
        let mut headers = Vec::new();
        if inner.changed {
            headers.push(store::store(&inner.stored, settings));
        }
        if inner.login_changed {
            match (&settings.login, &inner.login) {
                (Some((name, lifetime)), Some(value)) => {
                    headers.push(store::write(name, value.clone(), settings, Some(*lifetime)))
                }
                (Some((name, _)), None) => headers.push(store::delete(name, settings.secure)),
                (None, _) => tracing::warn!("set_login without a login cookie configured on the SessionLayer"),
            }
        }
        headers
    }
}

impl<T> std::fmt::Debug for Session<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Session(..)")
    }
}

impl<S: Send + Sync, T: SessionData> FromRequestParts<S> for Session<T> {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<Session<T>>().cloned().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "no SessionLayer for this session type",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flash_shows_once_and_tokens_stay() {
        let session = Session::new(0u32);
        assert_eq!(session.take_flash(), None);
        session.set_flash(Flash::Alert("no".into()));
        assert_eq!(session.take_flash(), Some(Flash::Alert("no".into())));
        assert_eq!(session.take_flash(), None);

        assert!(!session.verify_csrf(""));
        let token = session.csrf_token();
        assert_eq!(session.csrf_token(), token);
        assert!(session.verify_csrf(&token));
        assert!(!session.verify_csrf("forged"));
    }
}
