//! GET /muj-ucet: the account overview and the last ten orders.

use axum::extract::State;
use minijinja::context;
use serde::Serialize;

use super::{account_view, require_login};
use crate::AppState;
use crate::models::order::Order;
use crate::request::Req;
use crate::routes::Result;
use crate::views::{self, Page};

#[derive(Serialize)]
struct OrderCard {
    id: i64,
    date: String,
    items: usize,
    total: String,
}

pub async fn show(State(state): State<AppState>, req: Req) -> Result {
    let current = match require_login(&req) {
        Ok(current) => current,
        Err(redirect) => return Ok(*redirect),
    };
    let money = state.rates.money(req.locale).await;
    let orders: Vec<OrderCard> = Order::latest_of_user(&state.db, current.user.id, 10)
        .await?
        .iter()
        .map(|order| OrderCard {
            id: order.id,
            date: order.created_at.format("%d.%m.%Y").to_string(),
            items: order.items().len(),
            total: money.display(order.total_czk),
        })
        .collect();
    let account = account_view(&req, current);
    let page = Page::meta(&state, &req, "account");
    Ok(views::render(&state, &req, "accounts/show.html", page, context! { account, orders }).await)
}
