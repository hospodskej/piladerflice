//! GET /kosik: the cart page (checkout step 1).

use axum::extract::State;
use minijinja::context;

use crate::AppState;
use crate::request::Req;
use crate::routes::Result;
use crate::views::{self, Page};

pub async fn show(State(state): State<AppState>, req: Req) -> Result {
    let page = Page::meta(&state, &req, "cart");
    Ok(views::render(&state, &req, "carts/show.html", page, context! {}).await)
}
