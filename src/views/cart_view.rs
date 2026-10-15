//! The cart as the pages show it: in the header widget, on the cart page,
//! in the checkout summary and the "added to cart" modal.

use serde::Serialize;

use crate::AppState;
use crate::request::Req;
use crate::shop::cart::Cart;
use crate::shop::cart_line::CartLine;
use crate::shop::describe::Describer;
use crate::shop::without_vat;

#[derive(Clone, Debug, Serialize)]
pub struct LineView {
    pub id: String,
    pub description: String,
    pub image: Option<String>,
    pub quantity: i64,
    pub unit_price: String,
    pub total_excl_vat: String,
    pub total: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CartView {
    pub empty: bool,
    pub lines: Vec<LineView>,
    pub subtotal: String,
    pub vat: String,
    pub excl_vat: String,
}

/// The session's cart, described in the page's language and currency.
pub async fn load(state: &AppState, req: &Req) -> anyhow::Result<CartView> {
    let cart = req.session.read(|data| data.cart.clone());
    build(state, req, &cart).await
}

pub async fn build(state: &AppState, req: &Req, cart: &Cart) -> anyhow::Result<CartView> {
    let money = state.rates.money(req.locale).await;
    let lines = if cart.is_empty() {
        Vec::new()
    } else {
        let describer = Describer::load(&state.db, &state.translations).await?;
        cart.lines()
            .iter()
            .map(|line| line_view(line, &describer, req, &money))
            .collect()
    };
    let subtotal = cart.subtotal_czk();
    let excl_vat = cart.subtotal_excl_vat_czk();
    Ok(CartView {
        empty: cart.is_empty(),
        lines,
        subtotal: money.display(subtotal),
        vat: money.display(subtotal - excl_vat),
        excl_vat: money.display(excl_vat),
    })
}

/// One line, e.g. for the "added to cart" modal.
pub async fn line(state: &AppState, req: &Req, line: &CartLine) -> anyhow::Result<LineView> {
    let money = state.rates.money(req.locale).await;
    let describer = Describer::load(&state.db, &state.translations).await?;
    Ok(line_view(line, &describer, req, &money))
}

fn line_view(line: &CartLine, describer: &Describer, req: &Req, money: &crate::money::Money) -> LineView {
    LineView {
        id: line.id.clone(),
        description: describer.describe(line, req.locale),
        image: line.image.clone(),
        quantity: line.quantity,
        unit_price: money.display(line.unit_price_czk),
        total_excl_vat: money.display(without_vat(line.line_total_czk())),
        total: money.display(line.line_total_czk()),
    }
}
