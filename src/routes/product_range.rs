//! The product range pages (Sortiment), one per kind of wood. The products
//! they show live in the templates (`templates/home/sortiment/`), like they
//! did in the Rails views.

use axum::extract::State;
use axum::routing::{MethodRouter, get};
use minijinja::context;

use crate::AppState;
use crate::request::Req;
use crate::views::{self, Page};

/// The route for one page; `meta_key` names its texts and its template.
pub fn page(meta_key: &'static str) -> MethodRouter<AppState> {
    get(move |State(state): State<AppState>, req: Req| async move {
        let page = Page::meta(&state, &req, meta_key);
        views::render(
            &state,
            &req,
            &format!("home/sortiment/{meta_key}.html"),
            page,
            context! {},
        )
        .await
    })
}
