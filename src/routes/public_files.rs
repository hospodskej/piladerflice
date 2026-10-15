//! Files from Rails' `public/` folder (the site icons, the static error
//! pages), served at the top level like Rails does: `/icon.png`. Tried for
//! any URL no page answers, before the "page not found" page.

use axum::Extension;
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use super::not_found;
use crate::AppState;
use crate::request::{OriginalUri, Req};

/// The router's fallback: a public file if there is one, else the 404 page.
pub async fn or_not_found(
    State(state): State<AppState>,
    Extension(OriginalUri(uri)): Extension<OriginalUri>,
    req: Req,
) -> Response {
    match serve(&state, uri.path()).await {
        Some(response) => response,
        None => not_found::page(State(state), req).await,
    }
}

async fn serve(state: &AppState, path: &str) -> Option<Response> {
    let name = path.strip_prefix('/')?;
    // One plain file name: no folders, no "..", no hidden files.
    let plain = !name.is_empty()
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'));
    if !plain {
        return None;
    }
    let content_type = content_type(name)?;
    let bytes = tokio::fs::read(state.config.public_dir().join(name)).await.ok()?;
    let mut response = (StatusCode::OK, Body::from(bytes)).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    // Rails' production setting for public files.
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000"),
    );
    Some(response)
}

/// Only these kinds of files are served.
fn content_type(name: &str) -> Option<&'static str> {
    Some(match name.rsplit_once('.')?.1 {
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "ico" => "image/vnd.microsoft.icon",
        "html" => "text/html; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "webmanifest" => "application/manifest+json",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_known_kinds_of_files() {
        assert_eq!(super::content_type("icon.svg"), Some("image/svg+xml"));
        assert_eq!(super::content_type("secrets.yml"), None);
        assert_eq!(super::content_type("README"), None);
    }
}
