//! A variant's cart button, or a greyed-out "sold out".

use leptos::prelude::*;

use super::VariantView;
use crate::islands::cart::{AddToCart, BuyTexts};
use crate::ui::{CsrfField, t};

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
    view! {
        <AddToCart variant_id=variant_id show_qty=show_qty texts=BuyTexts::translated()>
            <CsrfField/>
            <input type="hidden" name="catalog_variant_id" id="catalog_variant_id" value=variant_id/>
        </AddToCart>
    }
}
