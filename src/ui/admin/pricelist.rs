//! /admin/pricelist_items: the price list of the contact page, one table
//! per category, each row its own little form (tied with `form=`), a row
//! for adding, and ↑/↓ for the order.

use leptos::prelude::*;

use super::form::{ActionButton, Submit, hidden_input, text_input};
use crate::admin::record::Values;
use crate::ui::{CsrfField, t};

pub struct PriceCategory {
    pub name: &'static str,
    /// The columns this category's rows fill in (each in CZ and DE).
    pub fields: &'static [&'static str],
    /// Its rows have an order (a category of one row doesn't).
    pub movable: bool,
    pub can_add: bool,
    pub items: Vec<(i64, Values)>,
}

fn title_key(category: &str) -> &'static str {
    match category {
        "stavebni" => "kontakt.cenik_stavebni_title",
        "palivove_volne" => "kontakt.cenik_palivove_volne_title",
        "palivove_skladane" => "kontakt.cenik_palivove_skladane_title",
        "zbytky" => "kontakt.cenik_zbytky_title",
        "kamenivo" => "kontakt.cenik_kamenivo_title",
        _ => "kontakt.cenik_sluzeb_title",
    }
}

/// A column's name in this category's table.
fn label(category: &str, field: &str) -> &'static str {
    let firewood = matches!(category, "palivove_volne" | "palivove_skladane");
    match (field, firewood, category) {
        ("subcategory", _, _) => "Dřevina",
        ("item_name", true, _) => "Délka",
        ("price", true, _) => "Cena za m³",
        ("item_name", _, "sluzby") => "Služba",
        ("item_name", _, _) => "Položka",
        ("details", _, _) => "Délky",
        _ => "Cena",
    }
}

const CONVERTED_PRICE_HINT: &str = "Německou cenu můžete nechat prázdnou – německá verze pak českou cenu automaticky přepočítá na eura podle aktuálního kurzu. Krychlový metr napište jako m³.";
const FIREWOOD_HINT: &str = "„/ m³“ se k ceně doplní automaticky. Německá cena se zobrazí přesně tak, jak ji zadáte (v eurech, bez přepočtu). Pro „není k dispozici“ zadejte ---. Položky se stejnou dřevinou se zobrazí v jedné tabulce.";

fn hint(category: &str) -> String {
    match category {
        "palivove_volne" | "palivove_skladane" => FIREWOOD_HINT.to_string(),
        "kamenivo" => format!("Zobrazuje se jedna cena. {CONVERTED_PRICE_HINT}"),
        _ => CONVERTED_PRICE_HINT.to_string(),
    }
}

#[component]
pub fn PricelistPage(categories: Vec<PriceCategory>) -> impl IntoView {
    view! {
        <div class="admin-toolbar">
            <h1 class="admin-page-title">"Ceník"</h1>
            <a target="_blank" rel="noopener" class="btn-outline-dark" href="/kontakt#cenik">"Zobrazit ceník na webu ↗"</a>
        </div>
        <p class="admin-page-subtitle">
            "Ceník sortimentu a služeb na stránce Kontakt. Změny se na webu projeví ihned po uložení."
        </p>
        {categories.into_iter().map(|category| view! { <Category category=category/> }).collect_view()}
    }
}

#[component]
fn Category(category: PriceCategory) -> impl IntoView {
    let name = category.name;
    let new_form = format!("new-pricelist-item-{name}-form");
    let item_form = |id: i64| format!("pricelist-item-{id}-form");
    let headers = category
        .fields
        .iter()
        .map(|field| {
            let label = label(name, field);
            view! {
                <th>{format!("{label} (CZ)")}</th>
                <th>{format!("{label} (DE)")}</th>
            }
        })
        .collect_view();
    let forms = category
        .items
        .iter()
        .map(|(id, _)| {
            view! {
                <form id=item_form(*id) action=format!("/admin/pricelist_items/{id}") accept-charset="UTF-8" method="post">
                    <CsrfField/>
                </form>
            }
        })
        .collect_view();
    let rows = category
        .items
        .into_iter()
        .map(|(id, values)| {
            let form = item_form(id);
            let cells = category
                .fields
                .iter()
                .map(|field| {
                    let label = label(name, field);
                    let de = format!("{field}_de");
                    view! {
                        <td>
                            {text_input(
                                field.to_string(),
                                values.get(field).map(str::to_string),
                                form.clone(),
                                None,
                                Some(format!("{label} (CZ)")),
                            )}
                        </td>
                        <td>
                            {text_input(
                                de.clone(),
                                values.get(&de).map(str::to_string),
                                form.clone(),
                                None,
                                Some(format!("{label} (DE)")),
                            )}
                        </td>
                    }
                })
                .collect_view();
            let url = format!("/admin/pricelist_items/{id}");
            view! {
                <tr>
                    <td class="admin-option-move">
                        {category
                            .movable
                            .then(|| {
                                view! {
                                    <ActionButton label="↑" action=format!("{url}/move_up") class="admin-move-btn" aria_label="Posunout nahoru"/>
                                    <ActionButton label="↓" action=format!("{url}/move_down") class="admin-move-btn" aria_label="Posunout dolů"/>
                                }
                            })}
                    </td>
                    {cells}
                    <td class="admin-option-actions">
                        <Submit label="Uložit" class="btn-outline-dark" form=form.clone()/>
                        <ActionButton
                            label="Smazat"
                            action=format!("{url}/delete")
                            class="admin-danger-btn"
                            confirm="Opravdu smazat tuto položku ceníku?"
                        />
                    </td>
                </tr>
            }
        })
        .collect_view();
    let new_row = category.can_add.then(|| {
        let cells = category
            .fields
            .iter()
            .map(|field| {
                let label = label(name, field);
                view! {
                    <td>
                        {text_input(
                            field.to_string(),
                            None,
                            new_form.clone(),
                            Some(format!("{label} (CZ)")),
                            Some(format!("Nová položka: {label} (CZ)")),
                        )}
                    </td>
                    <td>
                        {text_input(
                            format!("{field}_de"),
                            None,
                            new_form.clone(),
                            Some(format!("{label} (DE)")),
                            Some(format!("Nová položka: {label} (DE)")),
                        )}
                    </td>
                }
            })
            .collect_view();
        view! {
            <tr>
                <td>{hidden_input("category", name.to_string(), new_form.clone())}</td>
                {cells}
                <td>
                    <Submit label="+ Přidat" class="btn-primary" form=new_form.clone()/>
                </td>
            </tr>
        }
    });
    let new_form_tag = category.can_add.then(|| {
        view! {
            <form id=new_form.clone() action="/admin/pricelist_items" accept-charset="UTF-8" method="post">
                <CsrfField/>
            </form>
        }
    });
    view! {
        <div class="admin-box admin-option-group" id=name>
            <h2>{t(title_key(name))}</h2>
            <p class="admin-form-hint">{hint(name)}</p>
            {forms}
            {new_form_tag}
            <table class="admin-table">
                <thead>
                    <tr>
                        <th>"Pořadí"</th>
                        {headers}
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {rows}
                    {new_row}
                </tbody>
            </table>
        </div>
    }
}
