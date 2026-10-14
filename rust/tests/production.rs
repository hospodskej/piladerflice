//! What matters once the site is live (Phase 6): the protective headers,
//! the files of `public/`, the Rails image addresses still out there, and
//! the static error pages. The tests run with production settings
//! (`development: false`).

mod common;

use axum::http::{StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use common::{TestApp, get};

/// A Rails signed blob id for `blob_id` (the signature isn't checked).
fn signed_id(blob_id: i64) -> String {
    let data = STANDARD.encode(format!(r#"{{"_rails":{{"data":{blob_id},"pur":"blob_id"}}}}"#));
    format!("{data}--a998cdf96a244399b107aca7682e11e85555829a")
}

#[tokio::test]
async fn every_response_carries_rails_protective_headers() {
    let page = get("/").await;
    for (name, value) in [
        ("x-frame-options", "SAMEORIGIN"),
        ("x-xss-protection", "0"),
        ("x-content-type-options", "nosniff"),
        ("x-permitted-cross-domain-policies", "none"),
        ("referrer-policy", "strict-origin-when-cross-origin"),
        ("strict-transport-security", "max-age=63072000; includeSubDomains"),
        ("cache-control", "max-age=0, private, must-revalidate"),
    ] {
        assert_eq!(page.headers[name], value, "{name}");
    }
    // Responses with their own caching keep it.
    let icon = get("/icon.svg").await;
    assert_eq!(icon.headers[header::CACHE_CONTROL], "public, max-age=31536000");
    assert_eq!(icon.headers["x-content-type-options"], "nosniff");
}

#[tokio::test]
async fn files_of_public_are_served_at_the_top() {
    let icon = get("/icon.svg").await;
    assert_eq!(icon.status, StatusCode::OK);
    assert_eq!(icon.headers[header::CONTENT_TYPE], "image/svg+xml");
    assert_eq!(get("/icon.png").await.status, StatusCode::OK);
    let error_page = get("/500.html").await;
    assert_eq!(error_page.headers[header::CONTENT_TYPE], "text/html; charset=utf-8");

    // Only plain files of known kinds, and not under /at.
    for path in [
        "/at/icon.svg",
        "/.env",
        "/..%2Fconfig%2Fmaster.key",
        "/robots.yml",
        "/nothing.png",
    ] {
        assert_eq!(get(path).await.status, StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn rails_image_addresses_are_sent_on_for_good() {
    let app = TestApp::new().await;
    let id = signed_id(1);
    for path in [
        format!("/rails/active_storage/blobs/redirect/{id}/smrk%20foto.webp"),
        format!("/rails/active_storage/blobs/{id}/smrk%20foto.webp"),
        format!("/rails/active_storage/representations/redirect/{id}/eyJfcmFpbHMi--abc/smrk%20foto.webp"),
    ] {
        let page = app.get(&path).await;
        assert_eq!(page.status, StatusCode::PERMANENT_REDIRECT, "{path}");
        assert_eq!(
            page.location(),
            "/uploads/testkey0000000000000000000001/smrk%20foto.webp"
        );
    }
    let unknown = app
        .get(&format!(
            "/rails/active_storage/blobs/redirect/{}/x.webp",
            signed_id(999)
        ))
        .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    let garbage = app.get("/rails/active_storage/blobs/redirect/nonsense/x.webp").await;
    assert_eq!(garbage.status, StatusCode::NOT_FOUND);
}

/// Only pictures shown on the site anyway: a profile picture's blob id
/// leads nowhere, whatever the (unchecked) signature says.
#[tokio::test]
async fn private_pictures_are_not_reachable_through_old_addresses() {
    let app = TestApp::with_accounts().await;
    sqlx::raw_sql(
        "INSERT INTO active_storage_blobs (id, key, filename, content_type, metadata, service_name, byte_size, checksum, created_at)
           VALUES (77, 'avatarkey000000000000000000077', 'avatar.webp', 'image/webp', '{}', 'local', 1, 'x', '2026-01-01');
         INSERT INTO active_storage_attachments (name, record_type, record_id, blob_id, created_at)
           VALUES ('avatar', 'User', 1, 77, '2026-01-01');",
    )
    .execute(&app.state.db)
    .await
    .unwrap();
    let page = app
        .get(&format!(
            "/rails/active_storage/blobs/redirect/{}/avatar.webp",
            signed_id(77)
        ))
        .await;
    assert_eq!(page.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn refused_forms_get_the_static_error_page_of_public() {
    let app = TestApp::new().await;
    let mut browser = app.browser();
    browser.get("/eshop/smrk").await;
    let page = browser
        .post_raw("/cart_items", &[("catalog_variant_id", "10".into())])
        .await;
    assert_eq!(page.status, StatusCode::UNPROCESSABLE_ENTITY);
    page.assert_contains("The change you wanted was rejected");
}
