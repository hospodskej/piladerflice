//! A variant's cart button, or a greyed-out "sold out".

use leptos::prelude::*;

use super::VariantView;
use crate::ui::icons::CartIcon;
use crate::ui::{CsrfField, path, t};

/// `show_qty`: the quantity stepper before the button.
#[component]
pub fn BuyOrSoldOut(variant: VariantView, #[prop(default = true)] show_qty: bool) -> impl IntoView {
    if variant.in_stock {
        view! { <AddToCartForm variant_id=variant.id show_qty=show_qty/> }.into_any()
    } else {
        view! {
            <form class="button_to" method="post" action="#">
                <button
                    class="btn-primary btn-cart"
                    disabled="disabled"
                    style="background-color: #9CA3AF; color: #FFFFFF; border: none; cursor: not-allowed"
                    type="submit"
                >
                    {t("common.out_of_stock")}
                </button>
                <CsrfField/>
            </form>
        }
        .into_any()
    }
}

/// Puts a variant into the cart.
#[component]
pub fn AddToCartForm(variant_id: i64, show_qty: bool) -> impl IntoView {
    let add = t("common.add_to_cart");
    view! {
        <form class="add-to-cart-form" action=path("/cart_items") accept-charset="UTF-8" method="post">
            <CsrfField/>
            <input type="hidden" name="catalog_variant_id" id="catalog_variant_id" value=variant_id/>
            <div class="qty-cart-row">
                {if show_qty {
                    view! {
                        <div class="qty-stepper">
                            <button type="button" class="qty-btn qty-minus" aria-label=t("common.qty_decrease")>"−"</button>
                            <input type="number" name="quantity" id="quantity" value="1" min="1" class="qty-value" readonly="readonly"/>
                            <button type="button" class="qty-btn qty-plus" aria-label=t("common.qty_increase")>"+"</button>
                        </div>
                    }
                        .into_any()
                } else {
                    view! { <input type="hidden" name="quantity" id="quantity" value="1"/> }.into_any()
                }}
                <button type="submit" class="btn-primary btn-cart" title=add.clone() aria-label=add>
                    <CartIcon aria_hidden=true/>
                </button>
            </div>
        </form>
    }
}
