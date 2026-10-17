//! Step 2: delivery country, shipping and payment.

use leptos::prelude::*;

use super::{CartSummary, CountrySelect, Stepper};
use crate::shop::checkout::CheckoutState;
use crate::ui::{CsrfField, path, t, t_with};

#[component]
pub fn ShippingPage(checkout: CheckoutState, rate_per_km: String) -> impl IntoView {
    let country = checkout.get("country").unwrap_or("cz").to_string();
    let delivery = checkout.get("shipping_method") == Some("address");
    let bank_transfer = checkout.get("payment_method") == Some("bank_transfer");
    view! {
        <main class="main">
            <section class="checkout-section">
                <Stepper current=2/>
                <div class="checkout-layout">
                    <form class="checkout-main-col" action=path("/kosik/doprava") accept-charset="UTF-8" method="post">
                        <CsrfField/>
                        <div class="checkout-box">
                            <h3 class="checkout-box-title">{t("checkout.delivery_country_label")}</h3>
                            <div class="checkout-box-body">
                                <label for="country">{t("checkout.country_field")}</label>
                                <CountrySelect name="country" selected=country/>
                            </div>
                        </div>
                        <div class="checkout-box">
                            <h3 class="checkout-box-title">{t("checkout.shipping_method_label")}</h3>
                            <div class="checkout-box-body checkout-radio-list">
                                <Choice
                                    name="shipping_method"
                                    value="address"
                                    checked=delivery
                                    label=t("checkout.shipping_address")
                                    note=t_with("checkout.shipping_address_rate_note", &[("rate", &rate_per_km)])
                                />
                                <Choice name="shipping_method" value="pickup" checked=!delivery label=t("checkout.shipping_pickup") note=t("checkout.free")/>
                            </div>
                        </div>
                        <div class="checkout-box">
                            <h3 class="checkout-box-title">{t("checkout.payment_method_label")}</h3>
                            <div class="checkout-box-body checkout-radio-list">
                                <Choice name="payment_method" value="cash" checked=!bank_transfer label=t("checkout.payment_cash") note=t("checkout.free")/>
                                <Choice
                                    name="payment_method"
                                    value="bank_transfer"
                                    checked=bank_transfer
                                    label=t("checkout.payment_bank_transfer")
                                    note=t("checkout.free")
                                />
                            </div>
                        </div>
                        <div class="checkout-nav">
                            <a class="btn-outline-dark" data-turbo="false" href=path("/kosik")>{t("checkout.back")}</a>
                            " "
                            <input type="submit" value=t("checkout.continue") class="btn-primary"/>
                        </div>
                    </form>
                    <CartSummary/>
                </div>
            </section>
        </main>
    }
}

/// A radio button with its label and a note on the right.
#[component]
fn Choice(name: &'static str, value: &'static str, checked: bool, label: String, note: String) -> impl IntoView {
    view! {
        <label class="checkout-radio-option">
            <span>
                <input type="radio" name=name value=value checked=checked/>
                " "
                {label}
            </span>
            " "
            <span class="checkout-radio-note">{note}</span>
        </label>
    }
}
