//! The Turbo Streams the cart answers with.

use minijinja::{Value, context};

use crate::AppState;
use crate::request::Req;
use crate::views::{fragment, turbo};

/// The header cart widget, with the current cart.
pub async fn widget(state: &AppState, req: &Req) -> anyhow::Result<String> {
    let html = fragment::render(state, req, "shared/cart_widget.html", context! {}).await?;
    Ok(turbo::stream("replace", "cart-widget", &html))
}

/// The cart table on the cart page.
pub async fn contents(state: &AppState, req: &Req) -> anyhow::Result<String> {
    let html = fragment::render(state, req, "carts/_contents.html", context! {}).await?;
    Ok(turbo::stream("replace", "cart-page-contents", &html))
}

/// The "added to cart" modal for one line.
pub async fn added_modal(state: &AppState, req: &Req, item: Value) -> anyhow::Result<String> {
    let html = fragment::render(state, req, "shared/cart_added_modal.html", context! { item }).await?;
    Ok(turbo::stream("update", "cart-added-modal", &html))
}
