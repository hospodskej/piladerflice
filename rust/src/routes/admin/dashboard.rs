//! /admin: the overview, with counts and the latest orders.

use axum::extract::State;
use chrono::Utc;
use minijinja::context;

use super::orders::order_row;
use crate::AppState;
use crate::admin::dashboard;
use crate::models::order::Order;
use crate::request::Req;
use crate::routes::Result;
use crate::views::admin_layout;

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let stats = dashboard::stats(&state.db, Utc::now().naive_utc()).await?;
    let recent: Vec<_> = Order::latest(&state.db, Some(5)).await?.iter().map(order_row).collect();
    let data = context! {
        product_count => stats.products,
        order_count => stats.orders,
        orders_this_month => stats.orders_this_month,
        orders_last_month => stats.orders_last_month,
        recent_orders => recent,
    };
    Ok(admin_layout::render(&state, &req, "admin/dashboard.html", data))
}
