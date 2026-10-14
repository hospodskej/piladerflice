//! Pages with fixed content: the legal pages and the product range pages
//! (Sortiment) of each kind of wood.

use axum::extract::State;
use axum::response::Response;
use axum::routing::{MethodRouter, get};
use minijinja::context;

use crate::AppState;
use crate::request::Req;
use crate::views::{self, Page};

pub async fn terms(State(state): State<AppState>, req: Req) -> Response {
    legal(&state, &req, "terms", "legal.terms_title", "legal.terms_content_html")
}

pub async fn cookies_policy(State(state): State<AppState>, req: Req) -> Response {
    legal(
        &state,
        &req,
        "cookies",
        "legal.cookies_title",
        "legal.cookies_content_html",
    )
}

pub async fn privacy_policy(State(state): State<AppState>, req: Req) -> Response {
    legal(
        &state,
        &req,
        "privacy",
        "legal.privacy_title",
        "legal.privacy_content_html",
    )
}

fn legal(state: &AppState, req: &Req, meta_key: &str, title_key: &str, content_key: &str) -> Response {
    let page = Page::meta(state, req, meta_key);
    views::render(
        state,
        req,
        "pages/legal.html",
        page,
        context! { title_key, content_key },
    )
}

/// A Sortiment page (one kind of wood). The products shown live in the
/// template itself, like they did in the Rails views.
pub fn sortiment(meta_key: &'static str) -> MethodRouter<AppState> {
    get(move |State(state): State<AppState>, req: Req| async move {
        let page = Page::meta(&state, &req, meta_key);
        views::render(
            &state,
            &req,
            &format!("home/sortiment/{meta_key}.html"),
            page,
            context! {},
        )
    })
}
