//! The order as it would be placed now: built from the checkout details
//! and the cart, with the delivery price when it can be calculated.

use crate::AppState;
use crate::locale::Locale;
use crate::models::order::{Order, country_name};
use crate::request::Req;
use crate::shop::cart_line::OrderedItem;
use crate::shop::delivery::DeliveryQuote;
use crate::shop::vat_in;
use crate::ui::checkout::{DeliveryView, OrderView};

pub struct Draft {
    pub order: Order,
    pub delivery: Option<DeliveryQuote>,
}

pub async fn build(state: &AppState, req: &Req) -> anyhow::Result<Draft> {
    let (cart, checkout) = req.session.read(|data| (data.cart.clone(), data.checkout.clone()));
    let delivery = DeliveryQuote::for_checkout(&checkout, &state.routes).await;

    let text = |field: &str| checkout.fields().get(field).cloned();
    let snapshot: Vec<OrderedItem> = cart.lines().iter().map(OrderedItem::from).collect();
    let subtotal = cart.subtotal_czk();
    let total = subtotal + delivery.map_or(0, |quote| quote.price_czk());

    let order = Order {
        first_name: text("first_name"),
        last_name: text("last_name"),
        email: text("email"),
        phone: text("phone"),
        newsletter_opt_in: checkout.flag("newsletter_opt_in"),
        billing_street: text("billing_street"),
        billing_city: text("billing_city"),
        billing_zip: text("billing_zip"),
        billing_country: text("billing_country"),
        company_purchase: checkout.flag("company_purchase"),
        company_name: text("company_name"),
        company_ico: text("company_ico"),
        company_dic: text("company_dic"),
        delivery_address_different: checkout.flag("delivery_address_different"),
        delivery_street: text("delivery_street"),
        delivery_city: text("delivery_city"),
        delivery_zip: text("delivery_zip"),
        delivery_country: text("delivery_country"),
        shipping_method: text("shipping_method"),
        payment_method: text("payment_method"),
        items_snapshot: serde_json::to_string(&snapshot)?,
        subtotal_czk: subtotal,
        delivery_distance_km: delivery.map(|quote| quote.round_trip_km()),
        delivery_czk: delivery.map(|quote| quote.price_czk()),
        vat_czk: vat_in(total),
        total_czk: total,
        locale: req.locale.code().to_string(),
        ..Order::default()
    };
    Ok(Draft { order, delivery })
}

/// The draft as the summary page shows it.
pub async fn view(state: &AppState, locale: Locale, draft: &Draft) -> (OrderView, Option<DeliveryView>) {
    let money = state.rates.money(locale).await;
    let order = &draft.order;
    let text = |value: &Option<String>| value.clone().unwrap_or_default();
    let view = OrderView {
        shipping_method: text(&order.shipping_method),
        payment_method: text(&order.payment_method),
        subtotal: money.display(order.subtotal_czk),
        total: money.display(order.total_czk),
        vat: money.display(order.vat_czk),
        without_vat: money.display(order.total_czk - order.vat_czk),
        full_name: order.full_name(),
        email: text(&order.email),
        phone: text(&order.phone),
        billing_street: text(&order.billing_street),
        billing_zip: text(&order.billing_zip),
        billing_city: text(&order.billing_city),
        billing_country: country_name(order.billing_country.as_deref(), locale),
        company_purchase: order.company_purchase,
        company_name: text(&order.company_name),
        company_ico: text(&order.company_ico),
        company_dic: text(&order.company_dic),
    };
    let delivery = draft.delivery.map(|quote| DeliveryView {
        price: money.display(quote.price_czk()),
        round_trip_km: quote.round_trip_km(),
    });
    (view, delivery)
}
