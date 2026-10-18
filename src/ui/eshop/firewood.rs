//! Firewood: a grid of containers (lengths across, amounts down) and a row
//! of crates, with buttons to show only one of them.

use leptos::prelude::*;

use super::VariantView;
use super::buy::BuyOrSoldOut;
use crate::islands::widgets::{FilterButtons, Filtered, FilteredRule};
use crate::ui::{t, t_html};

pub struct FirewoodVariants {
    /// The column headings ("33 cm", "50 cm"...).
    pub lengths: Vec<String>,
    pub rows: Vec<FirewoodRow>,
    pub crates: Vec<VariantView>,
}

pub struct FirewoodRow {
    /// "5", "10", "13 a 14"
    pub label: String,
    /// One cell per length; None when there is no such variant.
    pub cells: Vec<Option<VariantView>>,
}

#[component]
pub fn Firewood(variants: FirewoodVariants) -> impl IntoView {
    let FirewoodVariants { lengths, rows, crates } = variants;
    let per_m3 = t("common.per_m3");
    view! {
        <section class="inventory-section">
            <div class="inventory-filter">
                <span class="filter-label">{t("common.loose_type_label")}</span>
                " "
                <FilterButtons
                    class="filter-buttons".to_string()
                    options=vec![
                        ("all".to_string(), t("common.all")),
                        ("kontejner".to_string(), t("common.container")),
                        ("bedny".to_string(), t("common.crate")),
                    ]
                />
            </div>
            <hr class="inventory-separator main-separator"/>
            <Filtered shown_for=vec!["kontejner".to_string()] id="section-kontejner".to_string() class="inventory-group".to_string()>
                <h2 class="section-title" inner_html=t_html("eshop.firewood.variants_heading_html")></h2>
                <div class="matrix-grid">
                    <div class="matrix-cell empty"></div>
                    {lengths.into_iter().map(|length| view! { <div class="matrix-cell col-header">{length}</div> }).collect_view()}
                    {rows
                        .into_iter()
                        .map(|row| {
                            view! {
                                <div class="matrix-cell row-header">
                                    <span>{format!("{} {per_m3}", row.label)}</span>
                                </div>
                                {row
                                    .cells
                                    .into_iter()
                                    .map(|cell| {
                                        view! {
                                            <div class="matrix-cell">
                                                {cell.map(|variant| view! { <Card variant=variant image_alt=t("common.container")/> })}
                                            </div>
                                        }
                                    })
                                    .collect_view()}
                            }
                        })
                        .collect_view()}
                </div>
            </Filtered>
            <FilteredRule class="inventory-separator mid-separator".to_string()/>
            <Filtered shown_for=vec!["bedny".to_string()] id="section-bedny".to_string() class="inventory-group".to_string()>
                <h2 class="section-title" inner_html=t_html("eshop.firewood.crates_heading_html")></h2>
                <div class="matrix-grid-bedny">
                    <div class="matrix-cell empty"></div>
                    {crates
                        .into_iter()
                        .map(|variant| {
                            view! {
                                <div class="matrix-cell">
                                    <Card variant=variant image_alt=t("common.crate")/>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </Filtered>
        </section>
    }
}

/// A container or a crate.
#[component]
fn Card(variant: VariantView, image_alt: String) -> impl IntoView {
    let per_m3 = t("common.per_m3");
    let kind = if variant.stacked {
        t("common.stacked_variant")
    } else {
        t("common.loose_variant")
    };
    let amount = format!(
        "{} {per_m3}",
        variant.amount.map(|amount| amount.to_string()).unwrap_or_default()
    );
    view! {
        <div class="inventory-card">
            <img alt=image_alt class="inventory-img" src=variant.image.clone()/>
            <div class="inventory-specs">
                <div class="spec-labels">
                    <span>{t("common.spec_variant")}</span>
                    <span>{t("common.spec_length")}</span>
                    <span>{t("common.spec_amount")}</span>
                </div>
                <div class="spec-values">
                    <span class="spec-value">{kind}</span>
                    <span class="spec-value">{variant.length.clone()}</span>
                    <span class="spec-value">{amount}</span>
                </div>
            </div>
            <p class="inventory-price">{variant.price.clone()}</p>
            <BuyOrSoldOut variant=variant show_qty=false/>
        </div>
    }
}
