//! The headers Rails adds to every response, for the browser's protection:
//! no showing the site in another site's frame, no guessing file types,
//! only the site's address (not the page) passed on to other sites. In
//! production also HSTS: browsers use only HTTPS for two years.
//!
//! Pages are never cached by the browser (they change: the cart, the
//! flash), unless the response says otherwise (assets, uploads).

use axum::extract::{Request, State};
use axum::http::{HeaderName, HeaderValue, header};
use axum::middleware::Next;
use axum::response::Response;

use crate::AppState;

const ALWAYS: [(&str, &str); 5] = [
    ("x-frame-options", "SAMEORIGIN"),
    ("x-xss-protection", "0"),
    ("x-content-type-options", "nosniff"),
    ("x-permitted-cross-domain-policies", "none"),
    ("referrer-policy", "strict-origin-when-cross-origin"),
];

pub async fn add(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    for (name, value) in ALWAYS {
        headers
            .entry(HeaderName::from_static(name))
            .or_insert(HeaderValue::from_static(value));
    }
    headers
        .entry(header::CACHE_CONTROL)
        .or_insert(HeaderValue::from_static("max-age=0, private, must-revalidate"));
    if !state.config.development {
        headers
            .entry(header::STRICT_TRANSPORT_SECURITY)
            .or_insert(HeaderValue::from_static("max-age=63072000; includeSubDomains"));
    }
    response
}
