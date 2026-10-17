//! The Turbo Streams the cart answers with.

use leptos::prelude::*;

use crate::AppState;
use crate::request::Req;
use crate::ui::cart::{CartAddedModal, CartContents};
use crate::ui::layout::CartWidget;
use crate::ui::{self, Ui};
use crate::views::cart_view::{self, LineView};
use crate::views::turbo;

/// The header cart widget, with the current cart.
pub async fn widget(state: &AppState, req: &Req) -> anyhow::Result<String> {
    let cart = cart_view::load(state, req).await?;
    let html = ui::render::html(Ui::new(state, req), move || view! { <CartWidget cart=cart/> });
    Ok(turbo::stream("replace", "cart-widget", &html))
}

/// The cart table on the cart page.
pub async fn contents(state: &AppState, req: &Req) -> anyhow::Result<String> {
    let cart = cart_view::load(state, req).await?;
    let html = ui::render::html(Ui::new(state, req), move || view! { <CartContents cart=cart/> });
    Ok(turbo::stream("replace", "cart-page-contents", &html))
}

/// The "added to cart" modal for one line.
pub async fn added_modal(state: &AppState, req: &Req, item: LineView) -> anyhow::Result<String> {
    let html = ui::render::html(Ui::new(state, req), move || view! { <CartAddedModal item=item/> });
    Ok(turbo::stream("update", "cart-added-modal", &html))
}
