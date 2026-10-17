//! GET /kosik: the cart page (checkout step 1).

use axum::extract::State;
use leptos::prelude::*;

use crate::AppState;
use crate::request::Req;
use crate::routes::Result;
use crate::ui::{self, cart::CartPage};
use crate::views::Page;

pub async fn show(State(state): State<AppState>, req: Req) -> Result {
    let page = Page::meta(&state, &req, "cart");
    Ok(ui::render::page(&state, &req, page, || view! { <CartPage/> }).await)
}
