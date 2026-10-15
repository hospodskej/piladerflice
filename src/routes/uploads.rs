//! GET /uploads/{key}/{filename}: an uploaded image (see
//! [`storage`](crate::storage)).

use axum::body::Body;
use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::AppState;
use crate::storage;

/// Types a browser may show directly. Anything else (an SVG with a script
/// in it, HTML...) is offered as a download instead.
const INLINE_TYPES: [&str; 11] = [
    "image/png",
    "image/gif",
    "image/jpeg",
    "image/tiff",
    "image/bmp",
    "image/vnd.adobe.photoshop",
    "image/vnd.microsoft.icon",
    "image/webp",
    "image/avif",
    "image/heic",
    "image/heif",
];

/// GET /uploads/{key}/{filename}. The key never changes for a file, so it
/// can be cached for good.
pub async fn serve(State(state): State<AppState>, UrlPath((key, _filename)): UrlPath<(String, String)>) -> Response {
    if !storage::valid_key(&key) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let upload: Option<(String,)> = match sqlx::query_as("SELECT content_type FROM uploads WHERE key = ?")
        .bind(&key)
        .fetch_optional(&state.db)
        .await
    {
        Ok(upload) => upload,
        Err(error) => {
            tracing::error!("loading upload {key}: {error}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let Some((content_type,)) = upload else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(bytes) = storage::read(&state.config.storage_root, &key).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let inline = INLINE_TYPES.contains(&content_type.as_str());
    let mut response = Response::new(Body::from(bytes));
    let headers = response.headers_mut();
    let header_value =
        HeaderValue::from_str(&content_type).unwrap_or(HeaderValue::from_static("application/octet-stream"));
    headers.insert(
        header::CONTENT_TYPE,
        if inline {
            header_value
        } else {
            HeaderValue::from_static("application/octet-stream")
        },
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static(if inline { "inline" } else { "attachment" }),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    response
}
