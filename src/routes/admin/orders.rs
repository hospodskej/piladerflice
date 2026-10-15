//! /admin/orders: the list, one order, deleting.

use axum::extract::State;
use minijinja::{Value, context};

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
use crate::views::{admin_layout, redirect};

/// An order as the tables show it.
pub fn order_row(order: &Order) -> Value {
    context! {
        id => order.id,
        full_name => order.full_name(),
        email => &order.email,
        total => czk(order.total_czk),
        pickup => order.shipping_method.as_deref() == Some("pickup"),
        bank_transfer => order.payment_method.as_deref() == Some("bank_transfer"),
        date => date(&order.created_at),
    }
}

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let orders: Vec<_> = Order::latest(&state.db, None).await?.iter().map(order_row).collect();
    Ok(admin_layout::render(
        &state,
        &req,
        "admin/orders/index.html",
        context! { orders },
    ))
}

pub async fn show(State(state): State<AppState>, req: Req, RecordId(id): RecordId) -> Result {
    let Some(order) = Order::find(&state.db, id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    let describer = Describer::load(&state.db, &state.translations).await?;
    let items: Vec<Value> = order
        .items()
        .iter()
        .map(|line| {
            context! {
                description => describer.describe(line, Locale::Cs),
                quantity => line.quantity,
                unit_price => czk(line.unit_price_czk),
                total => czk(line.line_total_czk()),
            }
        })
        .collect();
    let delivery_label = state.translations.text(
        Locale::Cs,
        "order_mailer.delivery",
        &[("km", &order.delivery_distance_km.unwrap_or_default().to_string())],
    );
    let data = context! {
        order => context! {
            id => order.id,
            date => date(&order.created_at),
            full_name => order.full_name(),
            email => &order.email,
            phone => &order.phone,
            newsletter => order.newsletter_opt_in,
            billing_street => &order.billing_street,
            billing_zip => &order.billing_zip,
            billing_city => &order.billing_city,
            billing_country => country_name(order.billing_country.as_deref(), Locale::Cs),
            company_purchase => order.company_purchase,
            company_name => &order.company_name,
            company_ico => &order.company_ico,
            company_dic => &order.company_dic,
            delivery_address_different => order.delivery_address_different,
            delivery_street => &order.delivery_street,
            delivery_zip => &order.delivery_zip,
            delivery_city => &order.delivery_city,
            delivery_country => country_name(order.delivery_country.as_deref(), Locale::Cs),
            pickup => order.shipping_method.as_deref() == Some("pickup"),
            by_address => order.shipping_method.as_deref() == Some("address"),
            bank_transfer => order.payment_method.as_deref() == Some("bank_transfer"),
            german => order.locale == "de",
            delivery_quoted => order.delivery_quoted(),
            subtotal => czk(order.subtotal_czk),
            delivery_label => delivery_label,
            delivery => czk(order.delivery_czk.unwrap_or_default()),
            without_vat => czk(order.total_czk - order.vat_czk),
            vat => czk(order.vat_czk),
            total => czk(order.total_czk),
        },
        items => items,
    };
    Ok(admin_layout::render(&state, &req, "admin/orders/show.html", data))
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
