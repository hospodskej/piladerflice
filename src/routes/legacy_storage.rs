//! Image addresses of the Rails app (`/rails/active_storage/blobs/redirect/
//! <signed id>/smrk.webp`) still found in search engines, shared links and
//! old e-mails: sent on, for good, to the same file at `/uploads/...`.
//!
//! The signed id carries the blob's id (`{"_rails":{"data":4,...}}` in
//! base64) and Rails' signature. Instead of checking the signature, only
//! images shown on the site anyway (products, variants, promos) are sent
//! on, so this can't be used to find anything private, like a profile
//! picture.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::AppState;
use crate::models::upload::Upload;

/// Records whose images are public.
const PUBLIC_RECORDS: [&str; 3] = ["CatalogProduct", "CatalogVariant", "Promo"];

/// `/rails/active_storage/blobs[/redirect|/proxy]/{signed_id}/{*filename}`
pub async fn blob(State(state): State<AppState>, Path((signed_id, _filename)): Path<(String, String)>) -> Response {
    redirect(&state, &signed_id).await
}

/// `/rails/active_storage/representations[/redirect|/proxy]/{signed_id}/{variation}/{*filename}`:
/// a resized version; the original is close enough.
pub async fn representation(
    State(state): State<AppState>,
    Path((signed_id, _variation, _filename)): Path<(String, String, String)>,
) -> Response {
    redirect(&state, &signed_id).await
}

async fn redirect(state: &AppState, signed_id: &str) -> Response {
    let Some(blob_id) = blob_id(signed_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let upload: Option<Upload> = match sqlx::query_as(
        "SELECT a.record_id, b.key, b.filename, b.content_type
         FROM active_storage_blobs b JOIN active_storage_attachments a ON a.blob_id = b.id
         WHERE b.id = ? AND a.record_type IN (?, ?, ?) LIMIT 1",
    )
    .bind(blob_id)
    .bind(PUBLIC_RECORDS[0])
    .bind(PUBLIC_RECORDS[1])
    .bind(PUBLIC_RECORDS[2])
    .fetch_optional(&state.db)
    .await
    {
        Ok(upload) => upload,
        Err(error) => {
            tracing::error!("looking up blob {blob_id}: {error}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    match upload {
        Some(upload) => Redirect::permanent(&upload.url()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// The blob id inside a Rails signed id ("eyJfcmFpbHMi...--a998cd...").
fn blob_id(signed_id: &str) -> Option<i64> {
    let decoded = percent_decode(signed_id);
    let (data, _signature) = decoded.split_once("--")?;
    let json = STANDARD.decode(data).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&json).ok()?;
    let rails = value.get("_rails")?;
    (rails.get("pur")?.as_str()? == "blob_id")
        .then(|| rails.get("data")?.as_i64())
        .flatten()
}

fn percent_decode(text: &str) -> String {
    form_urlencoded::parse(format!("x={}", text.replace('+', "%2B")).as_bytes())
        .next()
        .map(|(_, value)| value.into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_blob_id_from_a_rails_signed_id() {
        let signed = "eyJfcmFpbHMiOnsiZGF0YSI6NCwicHVyIjoiYmxvYl9pZCJ9fQ==--a998cdf96a244399b107aca7682e11e85555829a";
        assert_eq!(blob_id(signed), Some(4));
        // As it appears in a URL.
        assert_eq!(
            blob_id(
                "eyJfcmFpbHMiOnsiZGF0YSI6NCwicHVyIjoiYmxvYl9pZCJ9fQ%3D%3D--a998cdf96a244399b107aca7682e11e85555829a"
            ),
            Some(4)
        );
        assert_eq!(blob_id("garbage"), None);
        // Another purpose is not a blob id.
        let other = STANDARD.encode(r#"{"_rails":{"data":4,"pur":"session"}}"#);
        assert_eq!(blob_id(&format!("{other}--00")), None);
    }
}
