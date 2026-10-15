//! What matters once the site is live: the protective headers, the files
//! of `public/`, and the static error pages. The tests run with production settings
//! (`development: false`).

mod common;

use axum::http::{StatusCode, header};
use common::{TestApp, get};

#[tokio::test]
async fn every_response_carries_protective_headers() {
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
