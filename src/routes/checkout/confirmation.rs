//! /kosik/dekujeme: the thank-you page after an order, with the order
//! number. It also reports the purchase to Google Analytics (once).

use axum::extract::State;
use leptos::prelude::*;

use crate::AppState;
use crate::models::order::Order;
use crate::request::Req;
use crate::routes::Result;
use crate::shop::describe::Describer;
use crate::ui::checkout::{ConfirmationPage, Purchase};
use crate::ui::{self};
use crate::views::{Page, redirect};

pub async fn show(State(state): State<AppState>, req: Req) -> Result {
    let last = req.session.read(|data| data.last_order_id);
    let Some(order) = (match last {
        Some(id) => Order::find(&state.db, id).await?,
        None => None,
    }) else {
        return Ok(redirect::to(&req.path_for("/kosik")));
    };

    let describer = Describer::load(&state.db, &state.translations).await?;
    let ga_items: Vec<serde_json::Value> = order
        .items()
        .iter()
        .map(|item| {
            let name = describer.describe(
                &crate::shop::cart_line::CartLine {
                    specs: Vec::new(),
                    ..item.clone()
                },
                req.locale,
            );
            serde_json::json!({
                "item_id": item.product_key,
                "item_name": name,
                "price": item.unit_price_czk as f64,
                "quantity": item.quantity,
            })
        })
        .collect();
    let items_json = serde_json::to_string(&ga_items)?;

    let page = Page::meta(&state, &req, "checkout_confirmation");
    let purchase = Purchase {
        order_id: order.id,
        total_czk: order.total_czk as f64,
        items_json,
    };
    Ok(ui::render::page(&state, &req, page, move || view! { <ConfirmationPage purchase/> }).await)
}
