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

pub async fn handle(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let cookies = request
        .headers()
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok());
    let session = Session::new(cookie_store::load(cookies, &state.cookie_key));
    request.extensions_mut().insert(session.clone());

    let mut response = next.run(request).await;

    if let Some(data) = session.changed() {
        let value = cookie_store::store(&data, &state.cookie_key, state.config.secure_cookies);
        if let Ok(value) = HeaderValue::from_str(&value) {
            response.headers_mut().append(header::SET_COOKIE, value);
        }
    }
    response
}
