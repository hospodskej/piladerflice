//! The checkout's islands: the terms box of the summary, and the report of
//! a finished order to Google Analytics.

use leptos::html;
use leptos::prelude::*;

/// "I agree with the terms": the order can't be sent without it. When it
/// is missing, the box turns red and the message under it shows (without
/// the WebAssembly, the browser says so itself). `terms` is HTML (it links
/// the terms).
#[island]
pub fn TermsAgreement(terms: String, error: String) -> impl IntoView {
    let invalid = RwSignal::new(false);
    let message = NodeRef::<html::P>::new();
    view! {
        <label class="custom-checkbox-label checkout-checkbox" class:checkout-checkbox-invalid=invalid>
            <input
                type="checkbox"
                name="terms_agreement"
                value="1"
                id="termsAgreement"
                required=true
                on:invalid=move |event| {
                    event.prevent_default();
                    invalid.set(true);
                    request_animation_frame(move || {
                        if let Some(message) = message.get_untracked() {
                            super::browser::scroll_to_center(&message);
                        }
                    });
                }
                on:change=move |event| {
                    if event_target_checked(&event) {
                        invalid.set(false);
                    }
                }
            />
            <span class="custom-checkbox"></span>
            " "
            <span inner_html=terms></span>
        </label>
        <p class="checkout-terms-error" id="termsError" node_ref=message hidden=move || !invalid.get()>{error}</p>
    }
}

/// Tells Google Analytics about the order, once (a reload of the thank-you
/// page doesn't count it again). `items` is the items as JSON.
#[island]
pub fn PurchaseReport(order_id: i64, total_czk: f64, items: String) -> impl IntoView {
    Effect::new(move || super::browser::report_purchase(order_id, total_czk, &items));
}
