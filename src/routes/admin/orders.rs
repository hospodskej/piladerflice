//! /admin/orders: the list, one order, deleting.

use axum::extract::State;
use axum::http::StatusCode;
use leptos::prelude::*;

use super::params::RecordId;
use super::{czk, date};
use crate::AppState;
use crate::forms::FormData;
use crate::locale::Locale;
use crate::models::order::{Order, country_name};
use crate::request::Req;
use crate::routes::{Result, not_found};
use crate::session::Flash;
use crate::shop::describe::Describer;
use crate::ui::admin::{
    self,
    orders::{Address, ItemRow, OrderDetail, OrderPage, OrderRow, OrdersPage},
};
use crate::views::redirect;

/// An order as the tables show it.
pub fn order_row(order: &Order) -> OrderRow {
    OrderRow {
        id: order.id,
        full_name: order.full_name(),
        email: order.email.clone(),
        total: czk(order.total_czk),
        pickup: order.shipping_method.as_deref() == Some("pickup"),
        bank_transfer: order.payment_method.as_deref() == Some("bank_transfer"),
        date: date(&order.created_at),
    }
}

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let orders: Vec<_> = Order::latest(&state.db, None).await?.iter().map(order_row).collect();
    Ok(admin::page(
        &state,
        &req,
        StatusCode::OK,
        move || view! { <OrdersPage orders/> },
    ))
}

pub async fn show(State(state): State<AppState>, req: Req, RecordId(id): RecordId) -> Result {
    let Some(order) = Order::find(&state.db, id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    let describer = Describer::load(&state.db, &state.translations).await?;
    let items = order
        .items()
        .iter()
        .map(|line| ItemRow {
            description: describer.describe(line, Locale::Cs),
            quantity: line.quantity,
            unit_price: czk(line.unit_price_czk),
            total: czk(line.line_total_czk()),
        })
        .collect();
    let delivery_label = state.translations.text(
        Locale::Cs,
        "order_mailer.delivery",
        &[("km", &order.delivery_distance_km.unwrap_or_default().to_string())],
    );
    let address =
        |street: &Option<String>, zip: &Option<String>, city: &Option<String>, country: &Option<String>| Address {
            street: street.clone(),
            zip: zip.clone(),
            city: city.clone(),
            country: country_name(country.as_deref(), Locale::Cs),
        };
    let detail = OrderDetail {
        id: order.id,
        date: date(&order.created_at),
        full_name: order.full_name(),
        email: order.email.clone(),
        phone: order.phone.clone(),
        newsletter: order.newsletter_opt_in,
        billing: address(
            &order.billing_street,
            &order.billing_zip,
            &order.billing_city,
            &order.billing_country,
        ),
        company: order.company_purchase.then(|| {
            (
                order.company_name.clone(),
                order.company_ico.clone(),
                order.company_dic.clone(),
            )
        }),
        delivery_address: order.delivery_address_different.then(|| {
            address(
                &order.delivery_street,
                &order.delivery_zip,
                &order.delivery_city,
                &order.delivery_country,
            )
        }),
        pickup: order.shipping_method.as_deref() == Some("pickup"),
        by_address: order.shipping_method.as_deref() == Some("address"),
        bank_transfer: order.payment_method.as_deref() == Some("bank_transfer"),
        german: order.locale == "de",
        delivery: order
            .delivery_quoted()
            .then(|| (delivery_label, czk(order.delivery_czk.unwrap_or_default()))),
        subtotal: czk(order.subtotal_czk),
        without_vat: czk(order.total_czk - order.vat_czk),
        vat: czk(order.vat_czk),
        total: czk(order.total_czk),
        items,
    };
    Ok(admin::page(
        &state,
        &req,
        StatusCode::OK,
        move || view! { <OrderPage order=detail/> },
    ))
}

/// POST /admin/orders/{id}/delete (the "Smazat" buttons).
pub async fn delete(State(state): State<AppState>, req: Req, RecordId(id): RecordId, _form: FormData) -> Result {
    if Order::find(&state.db, id).await?.is_none() {
        return Ok(not_found::page(State(state), req).await);
    }
    Order::delete(&state.db, id).await?;
    req.session.set_flash(Flash::Notice("Objednávka byla smazána.".into()));
    Ok(redirect::to("/admin/orders"))
}
