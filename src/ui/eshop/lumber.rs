//! Lumber: variant cards, with buttons to filter them by dimension (or by
//! grade when the variants have no dimensions).

use leptos::prelude::*;

use super::VariantView;
use super::buy::BuyOrSoldOut;
use crate::islands::widgets::{FilterButtons, Filtered};
use crate::ui::{t, t_html};

pub struct LumberVariants {
    pub has_dimensions: bool,
    pub filters: Vec<FilterOption>,
    pub cards: Vec<VariantView>,
}

/// A filter button: the value cards are matched by, and its label.
pub struct FilterOption {
    pub key: String,
    pub label: String,
}

#[component]
pub fn Lumber(variants: LumberVariants, title: String) -> impl IntoView {
    let LumberVariants {
        has_dimensions,
        filters,
        cards,
    } = variants;
    let price_suffix = t_html("common.price_per_m3_html");
    view! {
        <section class="inventory-section">
            <hr class="inventory-separator main-separator"/>
            <h2 class="section-title" inner_html=t_html("eshop.lumber.variants_heading_html")></h2>
            {(filters.len() > 1)
                .then(|| {
                    view! {
                        <div class="inventory-filter">
                            <span class="filter-label">
                                {if has_dimensions { t("common.spec_dimension") } else { t("common.spec_grade") }}
                            </span>
                            " "
                            <FilterButtons
                                class="filter-buttons dimension-filter".to_string()
                                options=std::iter::once(("all".to_string(), t("common.all")))
                                    .chain(filters.into_iter().map(|option| (option.key, option.label)))
                                    .collect()
                            />
                        </div>
                    }
                })}
            <div class="simple-grid-3" id="lumberGrid">
                {cards
                    .into_iter()
                    .map(|variant| view! { <Card variant=variant title=title.clone() price_suffix=price_suffix.clone()/> })
                    .collect_view()}
            </div>
        </section>
    }
}

#[component]
fn Card(variant: VariantView, title: String, price_suffix: String) -> impl IntoView {
    let mm = |value: Option<i64>| value.map(|value| format!("{value} mm"));
    let shown_for = vec![variant.filter_value.clone()];
    view! {
        <Filtered shown_for=shown_for class="inventory-card".to_string()>
            {variant.image.clone().map(|src| view! { <img alt=title class="inventory-img" src=src/> })}
            <div class="inventory-specs">
                <div class="spec-labels">
                    {variant.grade.is_some().then(|| view! { <span>{t("common.spec_grade")}</span> })}
                    <span>{t("common.spec_length")}</span>
                    {variant.width_mm.is_some().then(|| view! { <span>{t("common.spec_width")}</span> })}
                    {variant.height_mm.is_some().then(|| view! { <span>{t("common.spec_height")}</span> })}
                </div>
                <div class="spec-values">
                    {variant.grade.clone().map(|grade| view! { <span class="spec-value">{grade}</span> })}
                    <span class="spec-value">{variant.length.clone()}</span>
                    {mm(variant.width_mm).map(|width| view! { <span class="spec-value">{width}</span> })}
                    {mm(variant.height_mm).map(|height| view! { <span class="spec-value">{height}</span> })}
                </div>
            </div>
            <p class="inventory-price" inner_html=format!("{} {price_suffix}", sapwood_i18n::escape_html(&variant.price))></p>
            <BuyOrSoldOut variant=variant/>
        </Filtered>
    }
}
