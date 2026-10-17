//! Step 4: everything once more, the total with delivery, the terms, and
//! the button that places the order.

use leptos::prelude::*;

use super::Stepper;
use crate::ui::render::current_cart;
use crate::ui::{CsrfField, path, t, t_html, t_with};

/// The order as it would be placed, in the page's language and currency.
pub struct OrderView {
    pub shipping_method: String,
    pub payment_method: String,
    pub subtotal: String,
    pub total: String,
    pub vat: String,
    pub without_vat: String,
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub billing_street: String,
    pub billing_zip: String,
    pub billing_city: String,
    pub billing_country: String,
    pub company_purchase: bool,
    pub company_name: String,
    pub company_ico: String,
    pub company_dic: String,
}

/// The delivery price, when it could be worked out.
pub struct DeliveryView {
    pub price: String,
    pub round_trip_km: i64,
}

/// `errors`: why the order wasn't placed (terms not accepted, details
/// missing).
#[component]
pub fn SummaryPage(order: OrderView, delivery: Option<DeliveryView>, errors: Vec<String>) -> impl IntoView {
    let link = |href: &str, key: &str| {
        format!(
            r#"<a target="_blank" rel="noopener" href="{}">{}</a>"#,
            path(href),
            sapwood_i18n::escape_html(&t(key))
        )
    };
    let terms = t_html("checkout.terms_agreement_html")
        .replace("{terms_link}", &link("/obchodni-podminky", "checkout.terms_link_text"))
        .replace(
            "{privacy_link}",
            &link("/zasady-ochrany-osobnich-udaju", "checkout.privacy_link_text"),
        );
    let address = order.shipping_method == "address";
    let shipping_note = if !address {
        view! { {t("checkout.free")} }.into_any()
    } else if let Some(delivery) = &delivery {
        view! { <strong>{delivery.price.clone()}</strong>" "{t("checkout.delivery_approximate")} }.into_any()
    } else {
        view! { {t("checkout.shipping_address_note")} }.into_any()
    };
    view! {
        <main class="main">
            <section class="checkout-section">
                <Stepper current=4/>
                {(!errors.is_empty())
                    .then(|| {
                        view! {
                            <div class="form-error">
                                <p>{t("checkout.submit_error")}</p>
                                <ul>{errors.into_iter().map(|message| view! { <li>{message}</li> }).collect_view()}</ul>
                            </div>
                        }
                    })}
                <div class="checkout-layout">
                    <form class="checkout-main-col" action=path("/kosik/souhrn") accept-charset="UTF-8" method="post">
                        <CsrfField/>
                        <Items/>
                        <Method
                            label=if address { t("checkout.shipping_address") } else { t("checkout.shipping_pickup") }
                            note=shipping_note
                        />
                        <Method
                            label=if order.payment_method == "bank_transfer" {
                                t("checkout.payment_bank_transfer")
                            } else {
                                t("checkout.payment_cash")
                            }
                            note=view! { {t("checkout.free")} }.into_any()
                        />
                        <div class="checkout-box">
                            <div class="checkout-box-body">
                                {delivery
                                    .map(|delivery| {
                                        view! {
                                            <div class="cart-total-row cart-total-excl-vat">
                                                <span>{t("checkout.goods_total")}</span>
                                                <span>{order.subtotal.clone()}</span>
                                            </div>
                                            <div class="cart-total-row cart-total-excl-vat">
                                                <span>{t_with("checkout.delivery_line", &[("km", &delivery.round_trip_km.to_string())])}</span>
                                                <span>{delivery.price}</span>
                                            </div>
                                        }
                                    })}
                                <div class="cart-total-row">
                                    <span>{t("cart.total_due")}</span>
                                    <strong>{order.total.clone()}</strong>
                                </div>
                                <div class="cart-total-row cart-total-excl-vat">
                                    <span>{t("cart.total_vat")}</span>
                                    <span>{order.vat.clone()}</span>
                                </div>
                                <div class="cart-total-row cart-total-excl-vat">
                                    <span>{t("cart.without_vat")}</span>
                                    <span>{order.without_vat.clone()}</span>
                                </div>
                            </div>
                        </div>
                        <label class="custom-checkbox-label checkout-checkbox">
                            <input type="checkbox" name="terms_agreement" value="1" id="termsAgreement"/>
                            <span class="custom-checkbox"></span>
                            " "
                            <span inner_html=terms></span>
                        </label>
                        <p class="checkout-terms-error" id="termsError" hidden=true>{t("checkout.terms_required")}</p>
                        <div class="checkout-nav">
                            <a class="btn-outline-dark" data-turbo="false" href=path("/kosik/udaje")>{t("checkout.back")}</a>
                            " "
                            <input type="submit" value=t("checkout.continue") class="btn-primary"/>
                        </div>
                    </form>
                    <Billing order=order/>
                </div>
            </section>
        </main>
    }
}

/// The cart lines with their prices.
#[component]
fn Items() -> impl IntoView {
    view! {
        <div class="checkout-box">
            <h3 class="checkout-box-title">{t("checkout.order_contents")}</h3>
            <div class="checkout-box-body">
                <ul class="checkout-summary-full-items">
                    {current_cart()
                        .lines
                        .into_iter()
                        .map(|line| {
                            view! {
                                <li class="checkout-summary-full-item">
                                    {line.image.map(|src| view! { <img alt=line.description.clone() class="cart-item-img" src=src/> })}
                                    <span class="cart-item-desc">{line.description.clone()}</span>
                                    <div class="checkout-summary-full-item-prices">
                                        <div class="cart-total-row cart-total-excl-vat">
                                            <span>{t("cart.table_price_excl_vat")}</span>
                                            <span>{line.total_excl_vat}</span>
                                        </div>
                                        <div class="cart-total-row">
                                            <span>{t("cart.table_price_incl_vat")}</span>
                                            <strong>{line.total}</strong>
                                        </div>
                                    </div>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </div>
        </div>
    }
}

/// The chosen shipping or payment, read-only.
#[component]
fn Method(label: String, note: AnyView) -> impl IntoView {
    view! {
        <div class="checkout-box checkout-method-summary">
            <div class="checkout-box-body checkout-radio-list">
                <div class="checkout-radio-option checkout-radio-option-readonly">
                    <span><span class="checkout-radio-dot"></span>" "{label}</span>
                    " "
                    <span class="checkout-radio-note">{note}</span>
                </div>
            </div>
        </div>
    }
}

/// Who pays, beside the form.
#[component]
fn Billing(order: OrderView) -> impl IntoView {
    view! {
        <aside class="checkout-summary-box">
            <h3 class="checkout-summary-title">{t("checkout.billing_details")}</h3>
            <p class="checkout-billing-summary">
                {order.full_name}
                <br/>
                <a href=format!("mailto:{}", order.email)>{order.email.clone()}</a>
                <br/>
                {format!("{}: {}", t("kontakt.form_telefon"), order.phone)}
            </p>
            <p class="checkout-billing-summary">
                {order.billing_street}
                <br/>
                {format!("{} {}", order.billing_zip, order.billing_city)}
                <br/>
                {order.billing_country}
            </p>
            {order
                .company_purchase
                .then(|| {
                    view! {
                        <p class="checkout-billing-summary">
                            {order.company_name}
                            <br/>
                            {format!("{}: {}", t("checkout.company_ico"), order.company_ico)}
                            <br/>
                            {format!("{}: {}", t("checkout.company_dic"), order.company_dic)}
                        </p>
                    }
                })}
        </aside>
    }
}
