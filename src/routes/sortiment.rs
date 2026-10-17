//! The Sortiment overview: one tile per kind of product.

use axum::extract::State;
use leptos::prelude::*;

use super::Result;
use super::components::category_tile;
use crate::AppState;
use crate::request::Req;
use crate::ui::{self, sortiment::SortimentPage};
use crate::views::Page;

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let tiles = category_tile::all(&state, &req).await?;
    let page = Page::meta(&state, &req, "sortiment");
    Ok(ui::render::page(&state, &req, page, move || view! { <SortimentPage tiles=tiles/> }).await)
}
