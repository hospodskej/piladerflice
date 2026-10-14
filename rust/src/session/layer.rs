//! Middleware around every request: load the session from its cookie, give
//! the handler a [`Session`] handle (in the request extensions), and when the
//! handler changed it, send the new cookie with the response. Requests that
//! don't touch the session (images, CSS) set no cookie.

use axum::extract::{Request, State};
use axum::http::{HeaderValue, header};
use axum::middleware::Next;
use axum::response::Response;

use super::{Session, cookie_store};
use crate::AppState;

/// The login cookie: the `sessions` row id, encrypted.
pub const LOGIN_COOKIE: &str = "_pila_login";
/// Like Rails' `cookies.permanent`: 20 years.
const LOGIN_LIFETIME: cookie::time::Duration = cookie::time::Duration::days(20 * 365);

pub async fn handle(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let cookies = request
        .headers()
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok());
    let login = cookie_store::read(cookies, &state.cookie_key, LOGIN_COOKIE).and_then(|id| id.parse().ok());
    let session = Session::with_login(cookie_store::load(cookies, &state.cookie_key), login);
    request.extensions_mut().insert(session.clone());

    let mut response = next.run(request).await;

    let secure = state.config.secure_cookies;
    let mut set_cookies = Vec::new();
    if let Some(data) = session.changed() {
        set_cookies.push(cookie_store::store(&data, &state.cookie_key, secure));
    }
    match session.changed_login() {
        Some(Some(id)) => set_cookies.push(cookie_store::write(
            LOGIN_COOKIE,
            id.to_string(),
            &state.cookie_key,
            secure,
            Some(LOGIN_LIFETIME),
        )),
        Some(None) => set_cookies.push(cookie_store::delete(LOGIN_COOKIE, secure)),
        None => {}
    }
    for value in set_cookies {
        if let Ok(value) = HeaderValue::from_str(&value) {
            response.headers_mut().append(header::SET_COOKIE, value);
        }
    }
    response
}
