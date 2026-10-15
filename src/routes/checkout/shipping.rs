//! /kosik/doprava: delivery country, shipping method and payment method.

use axum::extract::State;
use minijinja::context;

use super::guard::{self, Step};
use crate::AppState;
use crate::forms::FormData;
use crate::request::Req;
use crate::routes::Result;
use crate::shop::delivery::quote::CZK_PER_KM;
use crate::views::{self, Page, redirect};

pub async fn show(State(state): State<AppState>, req: Req) -> Result {
    if let Some(back) = guard::check(&req, Step::Shipping) {
        return Ok(back);
    }
    let checkout = req.session.read(|data| data.checkout.fields().clone());
    let rate_per_km = state.rates.money(req.locale).await.display(CZK_PER_KM);
    let page = Page::meta(&state, &req, "checkout_shipping");
    Ok(views::render(
        &state,
        &req,
        "checkout/shipping.html",
        page,
        context! { checkout, rate_per_km },
    )
    .await)
}

pub async fn update(State(_state): State<AppState>, req: Req, form: FormData) -> Result {
    if let Some(back) = guard::check(&req, Step::Shipping) {
        return Ok(back);
    }
    let allowed: Vec<(String, String)> = form
        .pairs()
        .iter()
        .filter(|(name, _)| ["shipping_method", "payment_method", "country"].contains(&name.as_str()))
        .cloned()
        .collect();
    req.session.update(|data| data.checkout.update(&allowed));
    Ok(redirect::to(&req.path_for("/kosik/udaje")))
}
