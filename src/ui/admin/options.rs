//! /admin/inquiry_form_options: the choices of the quote form's lists, one
//! table per list (each row its own little form); the first choice is
//! preselected.

use leptos::prelude::*;

use super::form::{ActionButton, Submit, hidden_input, text_input};
use crate::ui::CsrfField;

pub struct OptionCategory {
    pub name: &'static str,
    /// "Palivové dřevo"
    pub label: String,
    pub lists: Vec<OptionList>,
}

pub struct OptionList {
    pub field: &'static str,
    /// "Druh dřeva"
    pub label: String,
    /// (id, value, German value)
    pub choices: Vec<(i64, String, Option<String>)>,
}

#[component]
pub fn OptionsPage(categories: Vec<OptionCategory>) -> impl IntoView {
    view! {
        <div class="admin-toolbar">
            <h1 class="admin-page-title">"Formulář poptávky"</h1>
        </div>
        <p class="admin-page-subtitle">
            "Možnosti v rozbalovacích seznamech na kalkulačce v kontaktním formuláři. První možnost v seznamu je přednastavená (preset)."
        </p>
        {categories
            .into_iter()
            .map(|category| {
                let name = category.name;
                view! {
                    <h2 class="admin-section-title">{category.label}</h2>
                    {category.lists.into_iter().map(|list| view! { <List category=name list=list/> }).collect_view()}
                }
            })
            .collect_view()}
    }
}

#[component]
fn List(category: &'static str, list: OptionList) -> impl IntoView {
    let field = list.field;
    let new_form = format!("new-option-{category}-{field}-form");
    let option_form = |id: i64| format!("option-{id}-form");
    let forms = list
        .choices
        .iter()
        .map(|(id, _, _)| {
            view! {
                <form id=option_form(*id) action=format!("/admin/inquiry_form_options/{id}") accept-charset="UTF-8" method="post">
                    <CsrfField/>
                </form>
            }
        })
        .collect_view();
    let rows = list
        .choices
        .into_iter()
        .map(|(id, value, value_de)| {
            let form = option_form(id);
            let url = format!("/admin/inquiry_form_options/{id}");
            view! {
                <tr>
                    <td class="admin-option-move">
                        <ActionButton label="↑" action=format!("{url}/move_up") class="admin-move-btn"/>
                        <ActionButton label="↓" action=format!("{url}/move_down") class="admin-move-btn"/>
                    </td>
                    <td>{text_input("value".into(), Some(value), form.clone(), None, None)}</td>
                    <td>{text_input("value_de".into(), value_de, form.clone(), None, None)}</td>
                    <td class="admin-option-actions">
                        <Submit label="Uložit" class="btn-outline-dark" form=form.clone()/>
                        <ActionButton
                            label="Smazat"
                            action=format!("{url}/delete")
                            class="admin-danger-btn"
                            confirm="Opravdu smazat tuto možnost?"
                        />
                    </td>
                </tr>
            }
        })
        .collect_view();
    view! {
        <div class="admin-box admin-option-group" id=format!("{category}-{field}")>
            <h2>{list.label}</h2>
            {forms}
            <form id=new_form.clone() action="/admin/inquiry_form_options" accept-charset="UTF-8" method="post">
                <CsrfField/>
            </form>
            <table class="admin-table">
                <thead>
                    <tr>
                        <th>"Pořadí"</th>
                        <th>"Hodnota (CZ)"</th>
                        <th>"Hodnota (DE)"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {rows}
                    <tr>
                        <td></td>
                        <td>
                            {hidden_input("category", category.to_string(), new_form.clone())}
                            {hidden_input("field", field.to_string(), new_form.clone())}
                            {text_input("value".into(), None, new_form.clone(), Some("Nová možnost (CZ)".into()), None)}
                        </td>
                        <td>{text_input("value_de".into(), None, new_form.clone(), Some("Nová možnost (DE)".into()), None)}</td>
                        <td>
                            <Submit label="+ Přidat možnost" class="btn-primary" form=new_form.clone()/>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
    }
}
