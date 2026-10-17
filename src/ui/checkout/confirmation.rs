//! The thank-you page after an order, with the order number. It also
//! reports the purchase to Google Analytics (once per order).

use leptos::prelude::*;

use crate::ui::{asset, path, t, t_with};

/// The order as Google Analytics gets it.
pub struct Purchase {
    pub order_id: i64,
    /// "18000.0"
    pub total_czk: String,
    /// The items as JSON, safe inside `<script>`.
    pub items_json: String,
}

#[component]
pub fn ConfirmationPage(purchase: Purchase) -> impl IntoView {
    let script = format!(
        r#"(function () {{ var dedupeKey = "ga_purchase_fired_{id}"; try {{ if (sessionStorage.getItem(dedupeKey)) return; sessionStorage.setItem(dedupeKey, "1"); }} catch (e) {{}} if (typeof gtag === "function") {{ gtag("event", "purchase", {{ transaction_id: "{id}", value: {total}, currency: "CZK", items: {items} }}); }} }})();"#,
        id = purchase.order_id,
        total = purchase.total_czk,
        items = purchase.items_json,
    );
    view! {
        <main class="main">
            <section class="checkout-section checkout-confirmation">
                <div class="checkout-confirmation-box">
                    <svg viewBox="0 0 24 24" width="56" height="56" fill="none" stroke="#2C8FFF" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <circle cx="12" cy="12" r="10"/>
                        <polyline points="8 12 11 15 16 9"/>
                    </svg>
                    <h1>{t("checkout.confirmation_title")}</h1>
                    <p>{t_with("checkout.confirmation_text", &[("order_number", &purchase.order_id.to_string())])}</p>
                    <a class="btn-primary" href=path("/")>{t("checkout.back_to_homepage")}</a>
                </div>
                <img alt="" class="checkout-confirmation-mascot" width="320" height="320" src=asset("mascot/thumbs-up.webp")/>
            </section>
            <script inner_html=script></script>
        </main>
    }
}
