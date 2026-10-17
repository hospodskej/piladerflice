//! Machine-facing URLs: the health check and robots.txt.

use crate::request::Req;
use axum::http::header;
use axum::response::{Html, IntoResponse, Response};

/// GET /up: 200 when the app is running (for uptime monitors and the
/// reverse proxy).
pub async fn health() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html><html><body style="background-color: green"></body></html>"#)
}

/// GET /robots.txt. Made per request because the Sitemap line must be an
/// absolute URL, and the host is only known per request.
///
/// Private or per-visitor pages have nothing worth finding in a search
/// engine: admin, cart and checkout, the account, signing in. Everything
/// else stays crawlable, including /assets and the uploaded product photos,
/// which Google needs to render pages. The rules stay directly under
/// User-agent: some parsers end the group at a blank line.
pub async fn robots(req: Req) -> Response {
    const DISALLOWED: [&str; 9] = [
        "/admin",
        "/kosik",
        "/muj-ucet",
        "/prihlaseni",
        "/registrace",
        "/at/kosik",
        "/at/muj-ucet",
        "/at/prihlaseni",
        "/at/registrace",
    ];
    let rules: String = DISALLOWED.iter().map(|path| format!("Disallow: {path}\n")).collect();
    let text = format!(
        "# See https://www.robotstxt.org/robotstxt.html\nUser-agent: *\n{rules}\nSitemap: {}/sitemap.xml\n",
        req.base_url
    );
    (
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CACHE_CONTROL, "max-age=3600, public"),
        ],
        text,
    )
        .into_response()
}
