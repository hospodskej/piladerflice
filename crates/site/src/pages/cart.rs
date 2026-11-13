//! The cart page (step 1 of checkout).

use super::eshop::DeliveryNote;
use super::PageMeta;
use crate::cart_totals::TotalRows;
use crate::i18n::t;
use crate::layout::Layout;
use crate::visitor::visitor;
use leptos::prelude::*;

/// The steps at the top of the cart and checkout pages.
#[component]
pub fn CheckoutStepper(current: usize) -> impl IntoView {
    let l = visitor().locale;
    let steps = [t::checkout::step_cart(l), t::checkout::step_shipping(l), t::checkout::step_details(l), t::checkout::step_summary(l)];
    let count = steps.len();
    view! {
        <nav class="checkout-stepper" aria-label=t::checkout::step_summary(l)>
            {steps.into_iter().enumerate().map(|(i, label)| {
                let number = i + 1;
                let class = match number.cmp(&current) {
                    std::cmp::Ordering::Equal => "checkout-step current ",
                    std::cmp::Ordering::Less => "checkout-step  done",
                    std::cmp::Ordering::Greater => "checkout-step  ",
                };
                view! {
                    <div class=class>
                        <span class="checkout-step-number">{if number < current { "✓".to_owned() } else { number.to_string() }}</span>
                        <span class="checkout-step-label">{label}</span>
                    </div>
                    {(number != count).then(|| view! { <span class="checkout-step-arrow">"›"</span> })}
                }
            }).collect_view()}
        </nav>
    }
}

/// An auto-submitting quantity box (changing it saves the cart).
#[island]
fn QuantityBox(action: String, quantity: u32, label: String) -> impl IntoView {
    view! {
        <form action=action method="post">
            <input
                type="number"
                name="quantity"
                value=quantity
                min="1"
                step="1"
                class="qty-input"
                aria-label=label
                on:change=move |e| {
                    #[cfg(feature = "hydrate")]
                    {
                        use wasm_bindgen::JsCast;
                        if let Some(form) = e.target()
                            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                            .and_then(|i| i.form())
                        {
                            let _ = form.request_submit();
                        }
                    }
                    let _ = e;
                }
            />
        </form>
    }
}

#[component]
pub fn CartPage() -> impl IntoView {
    let v = visitor();
    let l = v.locale;
    #[cfg(feature = "ssr")]
    let (lines, totals, discount_reason) = {
        let reason = v.discount.as_ref().and_then(|d| {
            d.unusable_reason(v.cart.subtotal(), time::OffsetDateTime::now_utc().date())
                .map(|r| crate::forms::cart::discount_reason(&v, r, d.min_order))
        });
        (crate::cart_view::lines(&v), crate::cart_view::totals(&v, false), reason)
    };
    #[cfg(not(feature = "ssr"))]
    let (lines, totals, discount_reason): (Vec<crate::cart_view::LineView>, Vec<crate::cart_view::TotalRow>, Option<String>) = (vec![], vec![], None);

    let content = if v.cart.is_empty() {
        view! {
            <p class="cart-empty">{t::cart::empty(l)}</p>
            <a href=v.link("/eshop") class="btn-primary">{t::nav::eshop(l)}</a>
        }
        .into_any()
    } else {
        use t::cart as c;
        let rows = lines
            .into_iter()
            .map(|line| view! {
                <div class="checkout-cart-row">
                    <div class="col-item">
                        {line.image.clone().map(|src| view! { <img src=src alt=line.description.clone() class="cart-item-img"/> })}
                        <span class="cart-item-desc">{line.description.clone()}</span>
                    </div>
                    <div class="col-qty" data-label=c::table_quantity(l)>
                        <QuantityBox action=v.link(&format!("/cart_items/{}", line.variant_id)) quantity=line.quantity label=c::table_quantity(l).to_owned()/>
                    </div>
                    <div class="col-avail" data-label=c::table_availability(l)><span class="text-green">{t::common::in_stock(l)}</span></div>
                    <div class="col-unit-price" data-label=c::table_unit_price(l)>{line.unit_price}</div>
                    <div class="col-price-excl" data-label=c::table_price_excl_vat(l)>{line.total_excl_vat}</div>
                    <div class="col-price-incl" data-label=c::table_price_incl_vat(l)><strong>{line.total}</strong></div>
                    <div class="col-remove">
                        <form class="button_to" method="post" action={v.link(&format!("/cart_items/{}/odebrat", line.variant_id))}>
                            <button class="cart-remove-btn" aria-label=c::remove_aria(l) type="submit">"×"</button>
                        </form>
                    </div>
                </div>
            })
            .collect_view();
        let discount_box = match (&v.discount, &v.cart.discount_code) {
            (Some(code), Some(_)) => {
                let off = match code.kind {
                    domain::discount::DiscountKind::Percent => format!("{} %", code.value),
                    domain::discount::DiscountKind::Fixed => v.shop.price(l, domain::Czk(code.value)),
                };
                let note = match (&discount_reason, &v.discount_note) {
                    (Some(reason), _) => Some(view! {
                        <p class="discount-code-message discount-code-message-error">{c::discount_not_applied(l, &code.code, reason)}</p>
                    }.into_any()),
                    (None, Some((text, _))) => Some(view! { <p class="discount-code-message">{text.clone()}</p> }.into_any()),
                    _ => None,
                };
                view! {
                    <span class="discount-code-label">{c::discount_code_label(l)}</span>
                    <div class="discount-code-applied">
                        <span class="discount-code-chip">{format!("{} (−{off})", code.code)}</span>
                        <form class="button_to" method="post" action={v.link("/kosik/slevovy-kod/zrusit")}>
                            <button class="discount-code-remove" type="submit">{c::discount_remove(l)}</button>
                        </form>
                    </div>
                    {note}
                }
                .into_any()
            }
            _ => view! {
                <form action={v.link("/kosik/slevovy-kod")} method="post">
                    <label for="discount-code">{c::discount_code_label(l)}</label>
                    <div class="discount-code-row">
                        <input type="text" id="discount-code" name="code" autocomplete="off" autocapitalize="characters" placeholder=c::discount_code_placeholder(l) maxlength="40" required/>
                        <button type="submit" class="btn-primary">{c::discount_code_verify(l)}</button>
                    </div>
                </form>
                {v.discount_note.clone().map(|(text, error)| view! {
                    <p class=if error { "discount-code-message discount-code-message-error" } else { "discount-code-message" } role="alert">{text}</p>
                })}
            }
            .into_any(),
        };
        view! {
            <div class="checkout-cart-table">
                <div class="checkout-cart-row checkout-cart-header">
                    <span class="col-item">{c::table_contents(l)}</span>
                    <span class="col-qty">{c::table_quantity(l)}</span>
                    <span class="col-avail">{c::table_availability(l)}</span>
                    <span class="col-unit-price">{c::table_unit_price(l)}</span>
                    <span class="col-price-excl">{c::table_price_excl_vat(l)}</span>
                    <span class="col-price-incl">{c::table_price_incl_vat(l)}</span>
                    <span class="col-remove"></span>
                </div>
                {rows}
            </div>
            <div class="checkout-cart-footer">
                <div class="discount-code-box">{discount_box}</div>
                <div class="cart-totals checkout-cart-totals" id="cart-page-totals"><TotalRows rows=totals/></div>
            </div>
            <DeliveryNote/>
            <div class="checkout-nav">
                <a href=v.link("/eshop") class="btn-outline-dark">{c::back_to_shopping(l)}</a>
                <a href=v.link("/kosik/doprava") class="btn-primary">{t::checkout::r#continue(l)}</a>
            </div>
        }
        .into_any()
    };
    view! {
        <PageMeta title=t::meta::cart::title(l).to_owned()/>
        <Layout>
            <main class="main">
                <section class="checkout-section">
                    <CheckoutStepper current=1/>
                    <div id="cart-page-contents">{content}</div>
                </section>
            </main>
        </Layout>
    }
}
