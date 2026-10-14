//! The two e-mails of a new order: the confirmation for the customer (in
//! their language) and the notification for the business (in Czech).

use minijinja::{Value, context};
use serde::Serialize;

use super::Email;
use crate::AppState;
use crate::locale::Locale;
use crate::models::order::{Order, country_name};
use crate::money::Money;
use crate::shop::describe::Describer;

#[derive(Serialize)]
struct Line {
    description: String,
    quantity: i64,
    unit_price: String,
    line_total: String,
}

/// Thank-you e-mail to the customer.
pub async fn customer_confirmation(state: &AppState, order: &Order) -> anyhow::Result<Email> {
    let locale = Locale::from_code(&order.locale).unwrap_or(Locale::Cs);
    let money = state.rates.money(locale).await;
    let data = order_data(state, order, locale, &money).await?;
    let subject = match locale {
        Locale::De => format!("Bestellbestätigung Nr. {} – Pila Derflice", order.id),
        Locale::Cs => format!("Potvrzení objednávky č. {} – Pila Derflice", order.id),
    };
    render(
        state,
        "customer_confirmation",
        order.email.clone().unwrap_or_default(),
        subject,
        data,
    )
}

/// Notification of a new order for the business, always in Czech.
pub async fn new_order(state: &AppState, order: &Order) -> anyhow::Result<Email> {
    let money = state.rates.money(Locale::Cs).await;
    let data = order_data(state, order, Locale::Cs, &money).await?;
    let subject = format!("Nová objednávka #{} – {}", order.id, order.full_name());
    render(
        state,
        "new_order",
        state.mail_settings.order_notification_to.clone(),
        subject,
        data,
    )
}

async fn order_data(state: &AppState, order: &Order, locale: Locale, money: &Money) -> anyhow::Result<Value> {
    let describer = Describer::load(&state.db, &state.translations).await?;
    let lines: Vec<Line> = order
        .items()
        .iter()
        .map(|item| Line {
            description: describer.describe(item, locale),
            quantity: item.quantity,
            unit_price: money.display(item.unit_price_czk),
            line_total: money.display(item.line_total_czk()),
        })
        .collect();
    Ok(context! {
        locale => locale.code(),
        lines,
        id => order.id,
        first_name => text(&order.first_name),
        full_name => order.full_name(),
        email => text(&order.email),
        phone => text(&order.phone),
        newsletter => order.newsletter_opt_in,
        billing_street => text(&order.billing_street),
        billing_city => text(&order.billing_city),
        billing_zip => text(&order.billing_zip),
        billing_country => country_name(order.billing_country.as_deref(), locale),
        company_purchase => order.company_purchase,
        company_name => text(&order.company_name),
        company_ico => text(&order.company_ico),
        company_dic => text(&order.company_dic),
        delivery_address_different => order.delivery_address_different,
        delivery_street => text(&order.delivery_street),
        delivery_city => text(&order.delivery_city),
        delivery_zip => text(&order.delivery_zip),
        delivery_country => country_name(order.delivery_country.as_deref(), locale),
        shipping_method => text(&order.shipping_method),
        payment_method => text(&order.payment_method),
        delivery_quoted => order.delivery_quoted(),
        delivery_km => order.delivery_distance_km,
        delivery => order.delivery_czk.map(|czk| money.display(czk)),
        subtotal => money.display(order.subtotal_czk),
        total => money.display(order.total_czk),
        vat => money.display(order.vat_czk),
        without_vat => money.display(order.total_czk - order.vat_czk),
        created_at => order.created_at.format("%d.%m.%Y %H:%M").to_string(),
        customer_locale => &order.locale,
    })
}

fn text(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or_default()
}

/// Renders `mail/<name>.html` and `mail/<name>.txt`.
pub fn render(state: &AppState, name: &str, to: String, subject: String, data: Value) -> anyhow::Result<Email> {
    Ok(Email {
        to,
        subject,
        html: state.templates.render(&format!("mail/{name}.html"), data.clone())?,
        text: state.templates.render(&format!("mail/{name}.txt"), data)?,
    })
}
