//! The checkout: delivery and payment, details, summary, thank you. The
//! cart page is step 1 (`ui/cart.rs`).

mod confirmation;
mod details;
mod shipping;
mod summary;

pub use confirmation::{ConfirmationPage, Purchase};
pub use details::DetailsPage;
pub use shipping::ShippingPage;
pub use summary::{DeliveryView, OrderView, SummaryPage};

use leptos::prelude::*;

use super::render::current_cart;
use super::t;
use crate::shop::checkout::CheckoutState;

/// The four steps at the top; `current` is 1 to 4.
#[component]
pub fn Stepper(current: usize) -> impl IntoView {
    const STEPS: [&str; 4] = [
        "checkout.step_cart",
        "checkout.step_shipping",
        "checkout.step_details",
        "checkout.step_summary",
    ];
    view! {
        <nav class="checkout-stepper" aria-label=t("checkout.step_summary")>
            {STEPS
                .iter()
                .enumerate()
                .map(|(index, key)| {
                    let number = index + 1;
                    view! {
                        <div class="checkout-step" class:current=number == current class:done=number < current>
                            <span class="checkout-step-number">
                                {if number < current { "✓".to_string() } else { number.to_string() }}
                            </span>
                            " "
                            <span class="checkout-step-label">{t(key)}</span>
                        </div>
                        {(number < STEPS.len()).then(|| view! { " "<span class="checkout-step-arrow">"›"</span>" " })}
                    }
                })
                .collect_view()}
        </nav>
    }
}

/// The cart beside the checkout forms.
#[component]
fn CartSummary() -> impl IntoView {
    let cart = current_cart();
    view! {
        <aside class="checkout-summary-box">
            <h3 class="checkout-summary-title">{t("checkout.order_contents")}</h3>
            <ul class="checkout-summary-items">
                {cart
                    .lines
                    .into_iter()
                    .map(|line| {
                        view! {
                            <li class="checkout-summary-item">
                                {line
                                    .image
                                    .map(|src| view! { <img alt=line.description.clone() class="checkout-summary-item-img" src=src/> })}
                                <span class="checkout-summary-item-desc">{line.description.clone()}</span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
            <div class="cart-totals checkout-summary-totals">
                <div class="cart-total-row">
                    <span>{t("cart.total_due")}</span>
                    <strong>{cart.subtotal}</strong>
                </div>
                <div class="cart-total-row cart-total-excl-vat">
                    <span>{t("cart.total_vat")}</span>
                    <span>{cart.vat}</span>
                </div>
                <div class="cart-total-row cart-total-excl-vat">
                    <span>{t("cart.without_vat")}</span>
                    <span>{cart.excl_vat}</span>
                </div>
            </div>
        </aside>
    }
}

/// Czech Republic or Austria, with `selected` chosen.
#[component]
fn CountrySelect(name: &'static str, selected: String) -> impl IntoView {
    view! {
        <select name=name id=name>
            <option value="cz" selected=selected == "cz">{t("checkout.country_cz")}</option>
            <option value="at" selected=selected == "at">{t("checkout.country_at")}</option>
        </select>
    }
}

/// A field's value as entered so far.
fn value(checkout: &CheckoutState, field: &str) -> String {
    checkout.get(field).unwrap_or_default().to_string()
}
