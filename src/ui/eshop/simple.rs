//! Simple products (wood chips, sawdust, gravel...): one card per variant,
//! two or three in a row.

use leptos::prelude::*;

use super::VariantView;
use super::buy::BuyOrSoldOut;
use crate::ui::{t, t_html};

pub struct SimpleVariants {
    pub cards: Vec<VariantView>,
}

#[component]
pub fn Simple(variants: SimpleVariants, type_label: String) -> impl IntoView {
    let grid_class = if variants.cards.len() <= 2 {
        "simple-grid-2"
    } else {
        "simple-grid-3"
    };
    view! {
        <section class="inventory-section">
            <hr class="inventory-separator main-separator"/>
            <div id="section-kontejner" class="inventory-group">
                <h2 class="section-title" inner_html=t_html("common.available_variants_html")></h2>
                <div class=grid_class>
                    {variants
                        .cards
                        .into_iter()
                        .map(|variant| {
                            view! {
                                <div class="inventory-card">
                                    <img alt=t("common.container") class="inventory-img" src=variant.image.clone()/>
                                    <div class="inventory-specs">
                                        <div class="spec-labels">
                                            <span>{t("common.spec_description")}</span>
                                            <span>{t("common.spec_variant")}</span>
                                        </div>
                                        <div class="spec-values">
                                            <span class="spec-value">{type_label.clone()}</span>
                                            <span class="spec-value">{variant.label.clone()}</span>
                                        </div>
                                    </div>
                                    <p class="inventory-price">{variant.price.clone()}</p>
                                    <p
                                        class=if variant.in_stock { "spec-value text-green" } else { "spec-value text-red" }
                                        style="margin-bottom: 12px; font-size: 0.95rem"
                                    >
                                        {if variant.in_stock { t("common.in_stock") } else { t("common.out_of_stock") }}
                                    </p>
                                    <BuyOrSoldOut variant=variant/>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </section>
    }
}
