//! The cart page (checkout step 1), and the "added to cart" window.

use leptos::prelude::*;

use super::checkout::Stepper;
use super::render::current_cart;
use super::{CsrfField, path, t};
use crate::views::cart_view::{CartView, LineView};

#[component]
pub fn CartPage() -> impl IntoView {
    view! {
        <main class="main">
            <section class="checkout-section">
                <Stepper current=1/>
                <CartContents cart=current_cart()/>
            </section>
        </main>
    }
}

/// The cart table with the totals.
#[component]
pub fn CartContents(cart: CartView) -> impl IntoView {
    let content = if cart.empty {
        view! {
            <p class="cart-empty">{t("cart.empty")}</p>
            " "
            <a class="btn-primary" href=path("/eshop")>{t("nav.eshop")}</a>
        }
        .into_any()
    } else {
        view! {
            <div class="checkout-cart-table">
                <div class="checkout-cart-row checkout-cart-header">
                    <span class="col-item">{t("cart.table_contents")}</span>
                    <span class="col-qty">{t("cart.table_quantity")}</span>
                    <span class="col-avail">{t("cart.table_availability")}</span>
                    <span class="col-unit-price">{t("cart.table_unit_price")}</span>
                    <span class="col-price-excl">{t("cart.table_price_excl_vat")}</span>
                    <span class="col-price-incl">{t("cart.table_price_incl_vat")}</span>
                    <span class="col-remove"></span>
                </div>
                {cart.lines.into_iter().map(|line| view! { <Line line=line/> }).collect_view()}
            </div>
            <div class="checkout-cart-footer">
                <div class="discount-code-box">
                    <label for="discount-code">{t("cart.discount_code_label")}</label>
                    <div class="discount-code-row">
                        <input type="text" id="discount-code" name="discount-code" disabled=true/>
                        <button type="button" class="btn-primary" disabled=true>{t("cart.discount_code_verify")}</button>
                    </div>
                </div>
                <div class="cart-totals checkout-cart-totals">
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
            </div>
            <div class="checkout-nav">
                <a class="btn-outline-dark" data-turbo="false" href=path("/eshop")>{t("cart.back_to_shopping")}</a>
                " "
                <a class="btn-primary" href=path("/kosik/doprava")>{t("checkout.continue")}</a>
            </div>
        }
        .into_any()
    };
    view! { <div id="cart-page-contents">{content}</div> }
}

#[component]
fn Line(line: LineView) -> impl IntoView {
    view! {
        <div class="checkout-cart-row">
            <div class="col-item">
                {line.image.map(|src| view! { <img alt=line.description.clone() class="cart-item-img" src=src/> })}
                <span class="cart-item-desc">{line.description.clone()}</span>
            </div>
            <div class="col-qty">
                <form data-controller="auto-submit" action=path(&format!("/cart_items/{}", line.id)) accept-charset="UTF-8" method="post">
                    <CsrfField/>
                    <input
                        type="number"
                        name="quantity"
                        value=line.quantity
                        min="1"
                        step="1"
                        class="qty-input"
                        data-action="change->auto-submit#submit"
                        aria-label=t("cart.table_quantity")
                    />
                </form>
            </div>
            <div class="col-avail">
                <span class="text-green">{t("common.in_stock")}</span>
            </div>
            <div class="col-unit-price">{line.unit_price}</div>
            <div class="col-price-excl">{line.total_excl_vat}</div>
            <div class="col-price-incl">
                <strong>{line.total}</strong>
            </div>
            <div class="col-remove">
                <form class="button_to" method="post" action=path(&format!("/cart_items/{}/delete", line.id))>
                    <button class="cart-remove-btn" aria-label=t("cart.remove_aria") type="submit">"×"</button>
                    <CsrfField/>
                </form>
            </div>
        </div>
    }
}

/// Shown after "add to cart".
#[component]
pub fn CartAddedModal(item: LineView) -> impl IntoView {
    view! {
        <div
            class="cart-modal-backdrop"
            data-controller="cart-modal"
            data-action="click->cart-modal#backdropClick keydown@window->cart-modal#keydown"
        >
            <div class="cart-modal" role="dialog" aria-modal="true" data-cart-modal-target="dialog">
                <button type="button" class="cart-modal-close" aria-label=t("common.close") data-action="cart-modal#close">"✕"</button>
                <h3 class="cart-modal-title">{t("cart.added_title")}</h3>
                <hr class="cart-modal-divider"/>
                <div class="cart-modal-item">
                    {item.image.map(|src| view! { <img alt=item.description.clone() class="cart-modal-item-img" src=src/> })}
                    <div class="cart-modal-item-details">
                        <span class="cart-modal-item-desc">{item.description.clone()}</span>
                        " "
                        <span class="cart-modal-item-price">{item.total}</span>
                    </div>
                </div>
                <div class="cart-modal-actions">
                    <button type="button" class="btn-outline-dark" data-action="cart-modal#close">{t("cart.continue_shopping")}</button>
                    " "
                    <a class="btn-primary cart-modal-checkout" href=path("/kosik")>
                        {t("cart.to_checkout")}
                        " "
                        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <polyline points="9 18 15 12 9 6"/>
                        </svg>
                    </a>
                </div>
            </div>
        </div>
    }
}
