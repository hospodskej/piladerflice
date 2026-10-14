//! The Sortiment overview: one tile per kind of product.

use axum::extract::State;
use minijinja::context;

use super::Result;
use super::components::category_tile;
use crate::AppState;
use crate::request::Req;
use crate::views::{self, Page};

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let tiles = category_tile::all(&state, &req).await?;
    let page = Page::meta(&state, &req, "sortiment");
    Ok(views::render(
        &state,
        &req,
        "home/sortiment.html",
        page,
        context! { tiles },
    ))
}
