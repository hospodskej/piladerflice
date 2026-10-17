//! The legal pages: terms, cookie policy, privacy policy. Their text is one
//! long HTML translation each.

use axum::extract::State;
use axum::response::Response;
use leptos::prelude::*;

use crate::AppState;
use crate::request::Req;
use crate::ui::{self, legal::LegalPage};
use crate::views::Page;

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

async fn legal(
    state: &AppState,
    req: &Req,
    meta_key: &str,
    title_key: &'static str,
    content_key: &'static str,
) -> Response {
    let page = Page::meta(state, req, meta_key);
    ui::render::page(state, req, page, move || view! { <LegalPage title_key content_key/> }).await
}
