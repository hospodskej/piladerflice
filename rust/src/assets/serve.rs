//! GET /assets/*path: a static file. Fingerprinted URLs are cached for
//! good; the plain path still works (no-cache) for anything typed by hand.

use axum::body::Body;
use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::AppState;

pub async fn serve(State(state): State<AppState>, UrlPath(requested): UrlPath<String>) -> Response {
    let Some((file, digested)) = state.assets.find(&requested) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(bytes) = tokio::fs::read(&file).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mime = match file.extension().and_then(|ext| ext.to_str()) {
        Some("js") => "text/javascript".to_string(),
        _ => mime_guess::from_path(&file).first_or_octet_stream().to_string(),
    };
    let cache = if digested {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };

    let mut response = Response::new(Body::from(bytes));
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime).unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(cache));
    response
}
