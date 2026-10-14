//! The legal pages: terms, cookie policy, privacy policy. Their text is one
//! long HTML translation each.

use axum::extract::State;
use axum::response::Response;
use minijinja::context;

use crate::AppState;
use crate::request::Req;
use crate::views::{self, Page};

pub async fn terms(State(state): State<AppState>, req: Req) -> Response {
    legal(&state, &req, "terms", "legal.terms_title", "legal.terms_content_html").await
}

pub async fn cookies_policy(State(state): State<AppState>, req: Req) -> Response {
    legal(
        &state,
        &req,
        "cookies",
        "legal.cookies_title",
        "legal.cookies_content_html",
    )
    .await
}

pub async fn privacy_policy(State(state): State<AppState>, req: Req) -> Response {
    legal(
        &state,
        &req,
        "privacy",
        "legal.privacy_title",
        "legal.privacy_content_html",
    )
    .await
}

async fn legal(state: &AppState, req: &Req, meta_key: &str, title_key: &str, content_key: &str) -> Response {
    let page = Page::meta(state, req, meta_key);
    views::render(
        state,
        req,
        "pages/legal.html",
        page,
        context! { title_key, content_key },
    )
    .await
}
