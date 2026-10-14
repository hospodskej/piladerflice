//! Requests against the whole app, the way a browser would make them.

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use piladerflice::{AppState, Config, app};
use tower::ServiceExt;

use std::sync::LazyLock;

struct Page {
    status: StatusCode,
    headers: axum::http::HeaderMap,
    body: String,
}

/// Built once: fingerprinting all the images takes a moment.
static STATE: LazyLock<AppState> = LazyLock::new(|| AppState::new(Config::from_source_tree(false)).expect("app state"));

async fn get(path: &str) -> Page {
    let state = STATE.clone();
    let request = Request::get(path)
        .header(header::HOST, "piladerflice.test")
        .body(Body::empty())
        .unwrap();
    let response = app(state).oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    Page {
        status,
        headers,
        body: String::from_utf8_lossy(&bytes).into_owned(),
    }
}

fn assert_contains(page: &Page, needle: &str) {
    assert!(page.body.contains(needle), "expected {needle:?} in:\n{}", page.body);
}

const PUBLIC_PAGES: [&str; 8] = [
    "/obchodni-podminky",
    "/zasady-cookies",
    "/zasady-ochrany-osobnich-udaju",
    "/sortiment/palivove-drevo",
    "/sortiment/stavebni-rezivo",
    "/sortiment/truhlarske-rezivo",
    "/sortiment/okrasne-kamenivo",
    "/sortiment/vyrobni-zbytky",
];

#[tokio::test]
async fn every_page_renders_in_both_languages_without_missing_texts() {
    for path in PUBLIC_PAGES {
        for prefix in ["", "/at"] {
            let page = get(&format!("{prefix}{path}")).await;
            assert_eq!(page.status, StatusCode::OK, "{prefix}{path}");
            assert!(
                !page.body.contains("[missing:"),
                "{prefix}{path} has a missing translation"
            );
            assert_contains(
                &page,
                if prefix.is_empty() {
                    r#"<html lang="cs">"#
                } else {
                    r#"<html lang="de">"#
                },
            );
        }
    }
}

#[tokio::test]
async fn czech_page_has_title_description_and_language_links() {
    let page = get("/obchodni-podminky").await;
    assert_contains(&page, "<title>Obchodní podmínky | Pila Derflice</title>");
    assert_contains(
        &page,
        r#"<meta name="description" content="Obchodní podmínky e-shopu pily Derflice (Štěpán Merta).">"#,
    );
    assert_contains(
        &page,
        r#"<link rel="canonical" href="http://piladerflice.test/obchodni-podminky">"#,
    );
    assert_contains(
        &page,
        r#"<link rel="alternate" hreflang="de-AT" href="http://piladerflice.test/at/obchodni-podminky">"#,
    );
    assert_contains(
        &page,
        r#"<link rel="alternate" hreflang="x-default" href="http://piladerflice.test/obchodni-podminky">"#,
    );
    assert_contains(&page, r#"<meta property="og:locale" content="cs_CZ">"#);
    assert_contains(&page, r#"<meta name="twitter:card" content="summary_large_image">"#);
}

#[tokio::test]
async fn german_page_links_stay_in_german() {
    let page = get("/at/sortiment/palivove-drevo").await;
    assert_contains(&page, "Brennholz");
    assert_contains(&page, r#"href="/at/kontakt#cenik""#);
    assert_contains(&page, r#"href="/at/eshop?category=palivove&amp;""#);
    assert_contains(
        &page,
        r#"<a class="breadcrumb-item active" href="/at/sortiment/palivove-drevo">"#,
    );
    assert_contains(&page, r#"<meta property="og:locale" content="de_AT">"#);
    // The language switcher points at the same page in Czech.
    assert_contains(&page, r#"href="/sortiment/palivove-drevo""#);
}

#[tokio::test]
async fn product_rows_alternate_sides() {
    let page = get("/sortiment/stavebni-rezivo").await;
    assert_eq!(
        page.body
            .matches(r#"class="product-detail-hero reverse-layout""#)
            .count(),
        2
    );
    assert_eq!(page.body.matches(r#"class="product-detail-hero ""#).count(), 2);
}

#[tokio::test]
async fn unknown_pages_get_the_mascot_404_in_their_language() {
    let czech = get("/neexistuje").await;
    assert_eq!(czech.status, StatusCode::NOT_FOUND);
    assert_contains(&czech, "Stránka nenalezena | Pila Derflice");
    assert_contains(&czech, r#"<meta name="robots" content="noindex">"#);
    assert_contains(&czech, "/assets/mascot/");
    assert!(!czech.body.contains("breadcrumb-nav"));

    let german = get("/at/gibts-nicht").await;
    assert_eq!(german.status, StatusCode::NOT_FOUND);
    assert_contains(&german, r#"<html lang="de">"#);

    let file = get("/missing.png").await;
    assert_eq!(file.status, StatusCode::NOT_FOUND);
    assert!(file.body.is_empty());
}

#[tokio::test]
async fn old_language_links_redirect_permanently() {
    let page = get("/obchodni-podminky?locale=de&x=1").await;
    assert_eq!(page.status, StatusCode::MOVED_PERMANENTLY);
    assert_eq!(page.headers[header::LOCATION], "/at/obchodni-podminky?x=1");

    let page = get("/zasady-cookies?locale=cs").await;
    assert_eq!(page.headers[header::LOCATION], "/zasady-cookies");
}

#[tokio::test]
async fn machine_urls_exist_only_without_a_language() {
    assert_eq!(get("/up").await.status, StatusCode::OK);
    assert_eq!(get("/at/up").await.status, StatusCode::NOT_FOUND);

    let robots = get("/robots.txt").await;
    assert_eq!(robots.status, StatusCode::OK);
    assert!(robots.body.contains("Disallow: /at/kosik"));
    assert!(
        robots
            .body
            .trim_end()
            .ends_with("Sitemap: http://piladerflice.test/sitemap.xml")
    );
}

#[tokio::test]
async fn assets_are_fingerprinted_and_cached_for_good() {
    let page = get("/obchodni-podminky").await;
    let start = page.body.find("/assets/logo-wm-").expect("logo");
    let url = &page.body[start..start + page.body[start..].find('"').unwrap()];

    let logo = get(url).await;
    assert_eq!(logo.status, StatusCode::OK);
    assert_eq!(logo.headers[header::CONTENT_TYPE], "image/webp");
    assert_eq!(
        logo.headers[header::CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );

    assert_eq!(get("/assets/nope-12345678.css").await.status, StatusCode::NOT_FOUND);
    assert_contains(&page, r#""controllers": "/assets/controllers/index-"#);
    assert_contains(&page, r#"<link rel="stylesheet" href="/assets/base/global-"#);
}
