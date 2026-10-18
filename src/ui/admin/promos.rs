//! /admin/promos: the "Doporučujeme" promos on the home page, which take
//! turns week by week.

use leptos::prelude::*;

use super::form::{ActionButton, ErrorsBox, Form, Submit, file_field, text_field, text_field_with};
use super::products::group;
use crate::ui::CsrfField;

pub struct PromoRow {
    pub id: i64,
    pub title: Option<String>,
    pub price: Option<String>,
    pub link: Option<String>,
    /// Shown on the home page this week.
    pub this_week: bool,
}

#[component]
pub fn PromosPage(promos: Vec<PromoRow>) -> impl IntoView {
    let list = if promos.is_empty() {
        view! {
            <div class="admin-empty-state">
                "Zatím žádná doporučení. Bez záznamu se na hlavní stránce nezobrazí žádné doporučení."
            </div>
        }
        .into_any()
    } else {
        view! {
            <table class="admin-table">
                <thead>
                    <tr>
                        <th>"Název"</th>
                        <th>"Cena"</th>
                        <th>"Odkaz"</th>
                        <th>"Tento týden"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {promos
                        .into_iter()
                        .map(|promo| {
                            view! {
                                <tr>
                                    <td>
                                        <strong>{promo.title}</strong>
                                    </td>
                                    <td>{promo.price}</td>
                                    <td>{promo.link}</td>
                                    <td>{promo.this_week.then_some("✓ Aktivní")}</td>
                                    <td>
                                        <a href=format!("/admin/promos/{}/edit", promo.id)>"Upravit"</a>
                                        " "
                                        <ActionButton
                                            label="Smazat"
                                            action=format!("/admin/promos/{}/delete", promo.id)
                                            class="admin-danger-btn"
                                            confirm="Opravdu smazat tuto akční nabídku?"
                                        />
                                    </td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        }
        .into_any()
    };
    view! {
        <div class="admin-toolbar">
            <h1 class="admin-page-title">"Doporučujeme (hero)"</h1>
            <a class="btn-primary" href="/admin/promos/new">"+ Nové doporučení"</a>
        </div>
        <p class="admin-form-hint">
            "Nabídka na hlavní stránce se automaticky střídá každý týden mezi všemi níže uvedenými záznamy (podle pořadí vytvoření). Pro rotaci stačí mít alespoň 2 záznamy."
        </p>
        {list}
    }
}

pub struct PromoForm {
    pub id: Option<i64>,
    pub title: String,
    pub image: Option<String>,
    pub f: Form,
}

const FEATURES: [&str; 3] = ["feature_1", "feature_2", "feature_3"];
const FEATURES_DE: [&str; 3] = ["feature_1_de", "feature_2_de", "feature_3_de"];

#[component]
pub fn PromoPage(promo: PromoForm) -> impl IntoView {
    let PromoForm { id, title, image, f } = promo;
    let (heading, action) = match id {
        Some(id) => (format!("Upravit doporučení: {title}"), format!("/admin/promos/{id}")),
        None => ("Nové doporučení".to_string(), "/admin/promos".to_string()),
    };
    let features = |names: [&'static str; 3], de: bool| {
        names
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                let number = index + 1;
                let suffix = if de { "_de" } else { "" };
                view! {
                    <div class="form-group">
                        <label for=format!("promo_feature_{number}{suffix}")>
                            {format!("Vlastnost {number}{}", if de { " (DE)" } else { "" })}
                        </label>
                        {text_field(&f, name)}
                    </div>
                }
            })
            .collect_view()
    };
    view! {
        <a class="admin-back-link" href="/admin/promos">"← Zpět na přehled"</a>
        <h1 class="admin-page-title">{heading}</h1>
        <form class="admin-form" enctype="multipart/form-data" action=action accept-charset="UTF-8" method="post">
            <CsrfField/>
            <ErrorsBox f=f.clone()/>
            <div class="admin-form-grid">
                <div class="form-group">
                    <label for="image">"Obrázek"</label>
                    {image.map(|src| view! { <img class="admin-image-preview" src=src/> })}
                    {file_field(&f, "image", "image/*")}
                    <p class="admin-form-hint">
                        "Nahrajte novou fotku pro nahrazení aktuální. Necháte-li prázdné, zůstane aktuální obrázek."
                    </p>
                </div>
                {group("link", "Odkaz tlačítka", false, text_field_with(&f, "link", "/eshop/akat"), None)}
            </div>
            <h2 class="admin-section-title">"Český obsah"</h2>
            <div class="admin-form-grid">
                {group("title", "Název", true, text_field(&f, "title"), None)}
                {features(FEATURES, false)}
                {group("price", "Cena", false, text_field_with(&f, "price", "od 1500 / 1 PRMS"), None)}
            </div>
            <h2 class="admin-section-title">"Německý obsah"</h2>
            <p class="admin-form-hint" style="margin-bottom: 14px">"Pokud necháte prázdné, zobrazí se automaticky český text."</p>
            <div class="admin-form-grid">
                {group("title_de", "Název (DE)", true, text_field(&f, "title_de"), None)}
                {features(FEATURES_DE, true)}
                {group("price_de", "Cena (DE)", false, text_field_with(&f, "price_de", "ab 1500 / 1 Rm"), None)}
            </div>
            <div class="admin-form-actions">
                <Submit label="Uložit akční nabídku" class="btn-primary"/>
                <a class="btn-outline-dark" href="/admin/promos">"Zpět na přehled"</a>
            </div>
        </form>
    }
}
