//! /admin/orders: the list and one order.

use leptos::prelude::*;

use super::form::ActionButton;
use super::yes_no;

/// An order as the tables show it.
pub struct OrderRow {
    pub id: i64,
    pub full_name: String,
    pub email: Option<String>,
    /// "25 500 Kč"
    pub total: String,
    pub pickup: bool,
    pub bank_transfer: bool,
    /// "14.10.2026 18:28"
    pub date: String,
}

/// The orders; `full`: all columns and the delete buttons (the list), or
/// the short table of the overview.
#[component]
pub fn OrdersTable(orders: Vec<OrderRow>, full: bool) -> impl IntoView {
    view! {
        <table class="admin-table">
            <thead>
                <tr>
                    <th>"Č."</th>
                    <th>"Zákazník"</th>
                    {full.then(|| view! { <th>"E-mail"</th> })}
                    <th>"Celkem"</th>
                    {full.then(|| view! { <th>"Doprava"</th><th>"Platba"</th> })}
                    <th>"Datum"</th>
                    {full.then(|| view! { <th></th> })}
                </tr>
            </thead>
            <tbody>
                {orders
                    .into_iter()
                    .map(|order| {
                        view! {
                            <tr>
                                <td><a href=format!("/admin/orders/{}", order.id)>{format!("#{}", order.id)}</a></td>
                                <td>{order.full_name}</td>
                                {full.then(|| view! { <td>{order.email.clone()}</td> })}
                                <td>{order.total}</td>
                                {full
                                    .then(|| {
                                        view! {
                                            <td>{if order.pickup { "Osobní odběr" } else { "Doručení" }}</td>
                                            <td>{if order.bank_transfer { "Převod" } else { "Hotově" }}</td>
                                        }
                                    })}
                                <td>{order.date}</td>
                                {full
                                    .then(|| {
                                        view! {
                                            <td>
                                                <ActionButton
                                                    label="Smazat"
                                                    action=format!("/admin/orders/{}/delete", order.id)
                                                    class="admin-danger-btn"
                                                    confirm="Opravdu smazat tuto objednávku?"
                                                />
                                            </td>
                                        }
                                    })}
                            </tr>
                        }
                    })
                    .collect_view()}
            </tbody>
        </table>
    }
}

#[component]
pub fn OrdersPage(orders: Vec<OrderRow>) -> impl IntoView {
    view! {
        <h1 class="admin-page-title">"Objednávky"</h1>
        {if orders.is_empty() {
            view! { <p>"Zatím žádné objednávky."</p> }.into_any()
        } else {
            view! { <OrdersTable orders=orders full=true/> }.into_any()
        }}
    }
}

/// One order with everything the customer filled in.
pub struct OrderDetail {
    pub id: i64,
    pub date: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub newsletter: bool,
    pub billing: Address,
    /// Name, IČO, DIČ of a company purchase.
    pub company: Option<(Option<String>, Option<String>, Option<String>)>,
    pub delivery_address: Option<Address>,
    pub pickup: bool,
    pub by_address: bool,
    pub bank_transfer: bool,
    pub german: bool,
    /// "Doprava 25 km", "1 500 Kč" when the delivery was priced.
    pub delivery: Option<(String, String)>,
    pub subtotal: String,
    pub without_vat: String,
    pub vat: String,
    pub total: String,
    pub items: Vec<ItemRow>,
}

pub struct Address {
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub country: String,
}

pub struct ItemRow {
    pub description: String,
    pub quantity: i64,
    pub unit_price: String,
    pub total: String,
}

#[component]
fn AddressLines(address: Address) -> impl IntoView {
    view! {
        <p>
            {address.street}
            <br/>
            {address.zip}
            " "
            {address.city}
            <br/>
            {address.country}
        </p>
    }
}

#[component]
pub fn OrderPage(order: OrderDetail) -> impl IntoView {
    let id = order.id;
    let email = order.email;
    let mailto = format!("mailto:{}", email.clone().unwrap_or_default());
    let totals = match (order.delivery, order.by_address) {
        (Some((label, price)), _) => Some(
            view! {
                <p>{format!("Zboží: {}", order.subtotal)}</p>
                <p>{format!("{label}: {price}")}</p>
            }
            .into_any(),
        ),
        (None, true) => Some(
            view! { <p>"Doprava: vzdálenost se nepodařilo spočítat, cena dopravy není v celkové ceně."</p> }.into_any(),
        ),
        (None, false) => None,
    };
    view! {
        <a href="/admin/orders">"← Zpět na objednávky"</a>
        <div class="admin-toolbar">
            <div>
                <h1 class="admin-page-title">{format!("Objednávka #{id}")}</h1>
                <p class="admin-page-subtitle">{order.date}</p>
            </div>
            <ActionButton
                label="Smazat objednávku"
                action=format!("/admin/orders/{id}/delete")
                class="admin-danger-btn"
                confirm="Opravdu smazat tuto objednávku?"
            />
        </div>
        <div class="admin-detail-grid">
            <div class="admin-box">
                <h2>"Zákazník"</h2>
                <p>
                    {order.full_name}
                    <br/>
                    <a href=mailto>{email}</a>
                    <br/>
                    {order.phone}
                </p>
                <p>{format!("Newsletter: {}", yes_no(order.newsletter))}</p>
            </div>
            <div class="admin-box">
                <h2>"Fakturační adresa"</h2>
                <AddressLines address=order.billing/>
                {order
                    .company
                    .map(|(name, ico, dic)| {
                        view! {
                            <p>
                                <strong>{name}</strong>
                                <br/>
                                {format!("IČO: {}", ico.unwrap_or_default())}
                                <br/>
                                {format!("DIČ: {}", dic.unwrap_or_default())}
                            </p>
                        }
                    })}
            </div>
            {order
                .delivery_address
                .map(|address| {
                    view! {
                        <div class="admin-box">
                            <h2>"Doručovací adresa"</h2>
                            <AddressLines address=address/>
                        </div>
                    }
                })}
            <div class="admin-box">
                <h2>"Doprava a platba"</h2>
                <p>
                    {if order.pickup { "Osobní odběr na adrese skladu" } else { "Doručení na adresu" }}
                    <br/>
                    {if order.bank_transfer { "Bankovním převodem - proforma" } else { "Hotově při vyzvednutí" }}
                </p>
                <p>{format!("Jazyk zákazníka: {}", if order.german { "němčina" } else { "čeština" })}</p>
            </div>
        </div>
        <div class="admin-box">
            <h2>"Obsah objednávky"</h2>
            <table class="admin-table">
                <thead>
                    <tr>
                        <th>"Položka"</th>
                        <th>"Množství"</th>
                        <th>"Jedn. cena"</th>
                        <th>"Celkem"</th>
                    </tr>
                </thead>
                <tbody>
                    {order
                        .items
                        .into_iter()
                        .map(|item| {
                            view! {
                                <tr>
                                    <td>{item.description}</td>
                                    <td>{item.quantity}</td>
                                    <td>{item.unit_price}</td>
                                    <td>{item.total}</td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
            <div class="admin-order-totals">
                {totals}
                <p>{format!("Bez DPH: {}", order.without_vat)}</p>
                <p>{format!("DPH: {}", order.vat)}</p>
                <p>
                    <strong>{format!("Celkem k úhradě: {}", order.total)}</strong>
                </p>
            </div>
        </div>
    }
}
