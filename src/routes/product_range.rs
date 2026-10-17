//! The product range pages (Sortiment), one per kind of product. What they
//! show is fixed (`ui/sortiment.rs`).

use axum::extract::State;
use axum::routing::{MethodRouter, get};
use leptos::prelude::*;

use crate::AppState;
use crate::request::Req;
use crate::ui::{
    self,
    sortiment::{Range, RangePage},
};
use crate::views::Page;

/// The route for one page.
pub fn page(range: Range) -> MethodRouter<AppState> {
    get(move |State(state): State<AppState>, req: Req| async move {
        let page = Page::meta(&state, &req, range.meta_key());
        ui::render::page(&state, &req, page, move || view! { <RangePage range=range/> }).await
    })
}
