//! Machine-facing URLs: the health check and robots.txt.

use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use minijinja::context;

use crate::AppState;
use crate::request::Req;

/// GET /up: 200 when the app is running (for uptime monitors and the
/// reverse proxy), same page as Rails' health check.
pub async fn health() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html><html><body style="background-color: green"></body></html>"#)
}

/// GET /robots.txt. Rendered rather than a static file because the Sitemap
/// line must be an absolute URL, and the host is only known per request.
pub async fn robots(State(state): State<AppState>, req: Req) -> Response {
    match state.templates.render(
        "robots.txt",
        context! { sitemap_url => format!("{}/sitemap.xml", req.base_url) },
    ) {
        Ok(text) => (
            [
                (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                (header::CACHE_CONTROL, "max-age=3600, public"),
            ],
            text,
        )
            .into_response(),
        Err(error) => {
            tracing::error!("rendering robots.txt failed: {error:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
