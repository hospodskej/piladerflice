//! Redirects after a form ("Post/Redirect/Get"): 303 See Other, so the
//! browser (and Turbo) loads the next page with a GET.

use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};

pub fn to(location: &str) -> Response {
    (StatusCode::SEE_OTHER, [(header::LOCATION, location.to_string())]).into_response()
}
