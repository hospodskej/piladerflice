//! GET /muj-ucet: the account overview and the last ten orders.

use axum::extract::State;
use leptos::prelude::*;

use super::{account_view, require_login};
use crate::AppState;
use crate::models::order::Order;
use crate::request::Req;
use crate::routes::Result;
use crate::ui::account::{AccountPage, OrderCard};
use crate::ui::{self};
use crate::views::Page;

pub async fn show(State(state): State<AppState>, req: Req) -> Result {
    let current = match require_login(&state, &req) {
        Ok(current) => current,
        Err(redirect) => return Ok(*redirect),
    };
    let money = state.rates.money(req.locale).await;
    let orders: Vec<OrderCard> = Order::latest_of_user(&state.db, current.user.id, 10)
        .await?
        .iter()
        .map(|order| OrderCard {
            id: order.id,
            date: crate::czech_time::from_utc(order.created_at)
                .format("%d.%m.%Y")
                .to_string(),
            items: order.items().len(),
            total: money.display(order.total_czk),
        })
        .collect();
    let account = account_view(&req, current);
    let page = Page::meta(&state, &req, "account");
    Ok(ui::render::page(&state, &req, page, move || view! { <AccountPage account orders/> }).await)
}
