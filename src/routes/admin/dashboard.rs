//! /admin: the overview, with counts and the latest orders.

use axum::extract::State;
use axum::http::StatusCode;
use leptos::prelude::*;

use super::orders::order_row;
use crate::AppState;
use crate::admin::dashboard;
use crate::czech_time;
use crate::models::order::Order;
use crate::request::Req;
use crate::routes::Result;
use crate::ui::admin::{
    self,
    dashboard::{Dashboard, DashboardPage},
};

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let stats = dashboard::stats(&state.db, czech_time::now()).await?;
    let dashboard = Dashboard {
        products: stats.products,
        orders: stats.orders,
        orders_this_month: stats.orders_this_month,
        orders_last_month: stats.orders_last_month,
        recent: Order::latest(&state.db, Some(5)).await?.iter().map(order_row).collect(),
    };
    Ok(admin::page(
        &state,
        &req,
        StatusCode::OK,
        move || view! { <DashboardPage dashboard/> },
    ))
}
