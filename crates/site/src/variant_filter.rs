//! The "Filtr" panel on a product page: tick values, then Filtrovat. Within
//! a group any ticked value matches; across groups all must; a group with
//! nothing ticked doesn't filter. A card matches on its data-<name>
//! attribute. On the firewood grid, columns, rows and sections left empty
//! disappear; a plain card grid just closes up.

use crate::i18n::{Locale, t};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// (checkbox name, heading, options as (value, number shown in blue, unit)).
pub type FilterGroup = (String, String, Vec<(String, Option<String>, String)>);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterTexts {
    pub title: String,
    pub submit: String,
    pub reset: String,
    pub no_match: String,
}

pub fn filter_texts(l: Locale) -> FilterTexts {
    FilterTexts {
        title: t::eshop::filter::title(l).into(),
        submit: t::eshop::filter::submit(l).into(),
        reset: t::eshop::filter::reset(l).into(),
        no_match: t::eshop::filter::no_match(l).into(),
    }
}

#[island]
pub fn FilterPanel(groups: Vec<FilterGroup>, texts: FilterTexts) -> impl IntoView {
    let panel = NodeRef::<leptos::html::Details>::new();
    let nothing = RwSignal::new(false);
    let apply = move || {
        #[cfg(feature = "hydrate")]
        if let Some(panel) = panel.get_untracked() {
            nothing.set(browser::apply(&panel) == 0);
        }
    };
    let reset = move |_| {
        #[cfg(feature = "hydrate")]
        if let Some(panel) = panel.get_untracked() {
            browser::untick(&panel);
        }
        apply();
    };
    view! {
        <details class="variant-filter" node_ref=panel>
            <summary class="variant-filter-summary">
                <span>{texts.title.clone()}</span>
                <svg class="variant-filter-chevron" viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><polyline points="6 9 12 15 18 9"></polyline></svg>
            </summary>
            <form class="variant-filter-form" on:submit=move |e| { e.prevent_default(); apply(); }>
                {groups.into_iter().map(|(name, heading, options)| {
                    // A long list (fošny have ~17 sizes) is laid out in columns.
                    let class = if options.len() > 8 { "variant-filter-group variant-filter-group-wide" } else { "variant-filter-group" };
                    view! {
                        <fieldset class=class>
                            <legend>{heading}</legend>
                            <div class="variant-filter-options">
                                {options.into_iter().map(|(value, number, unit)| view! {
                                    <label class="variant-filter-option">
                                        <input type="checkbox" name=name.clone() value=value/>
                                        <span class="variant-filter-box" aria-hidden="true"></span>
                                        <span>{number.map(|n| view! { <span class="variant-filter-number">{n}</span>" " })}{unit}</span>
                                    </label>
                                }).collect_view()}
                            </div>
                        </fieldset>
                    }
                }).collect_view()}
                <div class="variant-filter-actions">
                    <button type="button" class="variant-filter-reset" on:click=reset>{texts.reset.clone()}</button>
                    <button type="submit" class="btn-primary variant-filter-submit">{texts.submit.clone()}</button>
                </div>
            </form>
        </details>
        <p class="variant-filter-empty" hidden=move || !nothing.get()>{texts.no_match.clone()}</p>
    }
}

#[cfg(feature = "hydrate")]
mod browser {
    use crate::islands::browser::all;
    use std::collections::{HashMap, HashSet};
    use wasm_bindgen::JsCast;
    use web_sys::{Element, HtmlElement, HtmlInputElement};

    fn style(el: &Element) -> Option<web_sys::CssStyleDeclaration> {
        el.clone().dyn_into::<HtmlElement>().ok().map(|e| e.style())
    }

    pub fn untick(panel: &Element) {
        for input in all(panel, "input[type=checkbox]") {
            if let Ok(input) = input.dyn_into::<HtmlInputElement>() {
                input.set_checked(false);
            }
        }
    }

    /// Filters the cards of the section around `panel`; returns how many show.
    pub fn apply(panel: &Element) -> usize {
        let Some(section) = panel.closest(".inventory-section").ok().flatten() else {
            return 0;
        };
        let mut chosen: HashMap<String, HashSet<String>> = HashMap::new();
        for input in all(panel, "input[type=checkbox]") {
            if let Ok(input) = input.dyn_into::<HtmlInputElement>()
                && input.checked()
            {
                chosen
                    .entry(input.name())
                    .or_default()
                    .insert(input.value());
            }
        }
        let matches = |card: &Element| {
            chosen.iter().all(|(name, values)| {
                values.contains(
                    &card
                        .get_attribute(&format!("data-{name}"))
                        .unwrap_or_default(),
                )
            })
        };
        let mut shown_total = 0;
        let sections = all(&section, "[data-filter-section]");
        for grid in all(&section, "[data-filter-grid]") {
            let cells = all(&grid, ".matrix-cell");
            if cells.is_empty() {
                for card in all(&grid, ".inventory-card") {
                    let shown = matches(&card);
                    if let Some(s) = style(&card) {
                        let _ = s.set_property("display", if shown { "" } else { "none" });
                    }
                    shown_total += usize::from(shown);
                }
                continue;
            }
            let (mut rows, mut columns) = (HashSet::new(), HashSet::new());
            for card in all(&grid, ".inventory-card") {
                let shown = matches(&card);
                if let Some(s) = style(&card) {
                    let _ = s.set_property("visibility", if shown { "" } else { "hidden" });
                }
                if shown {
                    if let Some(cell) = card.closest(".matrix-cell").ok().flatten() {
                        rows.insert(cell.get_attribute("data-row").unwrap_or_default());
                        columns.insert(cell.get_attribute("data-length").unwrap_or_default());
                    }
                    shown_total += 1;
                }
            }
            // A cell stays (maybe empty, to keep the grid lined up) while its
            // row and its column both still show something. The columns left
            // keep their full-grid width, so one length doesn't fill the page.
            for cell in cells {
                let keep = rows.contains(&cell.get_attribute("data-row").unwrap_or_default())
                    && columns.contains(&cell.get_attribute("data-length").unwrap_or_default());
                if let Some(s) = style(&cell) {
                    let _ = s.set_property("display", if keep { "" } else { "none" });
                }
            }
            let full: usize = grid
                .get_attribute("data-columns")
                .and_then(|c| c.parse().ok())
                .unwrap_or(columns.len());
            if let Some(s) = style(&grid) {
                let template = if columns.is_empty() {
                    String::new()
                } else {
                    format!(
                        "repeat({}, calc((100% - {} * 16px) / {full}))",
                        columns.len(),
                        full.saturating_sub(1)
                    )
                };
                let _ = s.set_property("grid-template-columns", &template);
            }
            if let Some(owner) = sections.iter().find(|s| s.contains(Some(&grid)))
                && let Some(s) = style(owner)
            {
                let _ = s.set_property("display", if columns.is_empty() { "none" } else { "" });
            }
        }
        let shown_sections = sections
            .iter()
            .filter(|s| {
                style(s)
                    .and_then(|st| st.get_property_value("display").ok())
                    .as_deref()
                    != Some("none")
            })
            .count();
        for separator in all(&section, "[data-filter-separator]") {
            if let Some(s) = style(&separator) {
                let _ = s.set_property("display", if shown_sections < 2 { "none" } else { "" });
            }
        }
        shown_total
    }
}
