//! Images uploaded in the admin (product photos, promo pictures). They are
//! stored the way Rails' Active Storage stores them: a row in
//! `active_storage_blobs` and the file at `storage/ab/cd/abcd…` (the key's
//! first two pairs of characters as folders).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::AppState;
use crate::db::Db;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Upload {
    pub record_id: i64,
    pub key: String,
    pub filename: String,
    pub content_type: Option<String>,
}

impl Upload {
    pub fn url(&self) -> String {
        let filename: String = form_urlencoded::byte_serialize(self.filename.as_bytes()).collect();
        format!("/uploads/{}/{}", self.key, filename.replace('+', "%20"))
    }
}

/// The `image` of every record of one kind ("CatalogProduct", "Promo", ...),
/// by record id. The tables are small, so loading them all is fine.
pub async fn images_of(db: &Db, record_type: &str) -> sqlx::Result<HashMap<i64, Upload>> {
    let uploads: Vec<Upload> = sqlx::query_as(
        "SELECT a.record_id, b.key, b.filename, b.content_type
         FROM active_storage_attachments a JOIN active_storage_blobs b ON b.id = a.blob_id
         WHERE a.record_type = ? AND a.name = 'image'",
    )
    .bind(record_type)
    .fetch_all(db)
    .await?;
    Ok(uploads.into_iter().map(|upload| (upload.record_id, upload)).collect())
}

/// Blob keys are 28 lowercase letters and digits; anything else is not a key
/// (and must never reach the file system).
fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= 64 && key.chars().all(|c| c.is_ascii_alphanumeric())
}

fn file_for(root: &Path, key: &str) -> PathBuf {
    root.join(&key[0..2]).join(&key[2..4]).join(key)
}

/// Types a browser may show directly (Active Storage's
/// content_types_allowed_inline). Anything else (an SVG with a script in it,
/// HTML...) is offered as a download instead.
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
    if !valid_key(&key) || key.len() < 4 {
        return StatusCode::NOT_FOUND.into_response();
    }
    let blob: Option<(Option<String>,)> =
        match sqlx::query_as("SELECT content_type FROM active_storage_blobs WHERE key = ?")
            .bind(&key)
            .fetch_optional(&state.db)
            .await
        {
            Ok(blob) => blob,
            Err(error) => {
                tracing::error!("loading blob {key}: {error}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
    let Some((content_type,)) = blob else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(bytes) = tokio::fs::read(file_for(&state.config.storage_root, &key)).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".into());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_plain_keys_are_accepted() {
        assert!(valid_key("vipnt6c82q5dy393de7xojdl6nbv"));
        assert!(!valid_key("../../etc/passwd"));
        assert!(!valid_key("ab/cd"));
        assert!(!valid_key(""));
    }

    #[test]
    fn files_live_in_two_levels_of_folders() {
        assert_eq!(
            file_for(Path::new("/s"), "vipnt6c8"),
            PathBuf::from("/s/vi/pn/vipnt6c8")
        );
    }

    #[test]
    fn urls_escape_the_filename() {
        let upload = Upload {
            record_id: 1,
            key: "abcd".into(),
            filename: "dub štípaný.webp".into(),
            content_type: None,
        };
        assert_eq!(upload.url(), "/uploads/abcd/dub%20%C5%A1t%C3%ADpan%C3%BD.webp");
    }
}
