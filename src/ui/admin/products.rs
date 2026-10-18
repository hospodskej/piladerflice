//! /admin/catalog_products: the catalogue, and a product's form with its
//! variants listed below.

use leptos::prelude::*;

use super::form::{
    ActionButton, ErrorsBox, Form, Submit, check_box, file_field, number_field, select, text_area, text_field,
};
use super::yes_no;
use crate::ui::CsrfField;

pub struct ProductRow {
    pub id: i64,
    pub title: String,
    pub key: String,
    pub template: String,
    pub category: String,
    pub variant_count: i64,
    /// "od 1 500 Kč"; None when no variant has a price.
    pub starting_price: Option<String>,
    pub active: bool,
}

#[component]
pub fn ProductsPage(products: Vec<ProductRow>) -> impl IntoView {
    let list = if products.is_empty() {
        view! { <div class="admin-empty-state">"Zatím žádné produkty v katalogu."</div> }.into_any()
    } else {
        view! {
            <table class="admin-table">
                <thead>
                    <tr>
                        <th>"Produkt"</th>
                        <th>"Šablona"</th>
                        <th>"Kategorie"</th>
                        <th>"Varianty"</th>
                        <th>"Od ceny"</th>
                        <th>"Aktivní"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {products
                        .into_iter()
                        .map(|product| {
                            view! {
                                <tr>
                                    <td>
                                        <strong>{product.title}</strong>
                                        <br/>
                                        <span style="color:#999; font-size:0.8rem">{product.key}</span>
                                    </td>
                                    <td>{product.template}</td>
                                    <td>{product.category}</td>
                                    <td>{product.variant_count}</td>
                                    <td>
                                        {match product.starting_price {
                                            Some(price) => price.into_any(),
                                            None => view! { <span style="color:#D33">"bez ceny"</span> }.into_any(),
                                        }}
                                    </td>
                                    <td>{yes_no(product.active)}</td>
                                    <td>
                                        <a href=format!("/admin/catalog_products/{}/edit", product.id)>"Upravit"</a>
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
            <h1 class="admin-page-title">"Katalog a ceny"</h1>
            <a class="btn-primary" href="/admin/catalog_products/new">"+ Nový produkt"</a>
        </div>
        {list}
    }
}

/// A new product (`id` None) or an existing one with its variants.
pub struct ProductForm {
    pub id: Option<i64>,
    pub title: String,
    /// The current picture's URL.
    pub image: Option<String>,
    pub variants: Vec<VariantRow>,
    pub f: Form,
}

/// A variant in the product's table.
pub struct VariantRow {
    pub id: i64,
    pub image: Option<String>,
    /// "AB / 200×50 mm", or the variant's label.
    pub description: String,
    /// "Sypané (kontejner)"
    pub group: Option<&'static str>,
    pub length_label: Option<String>,
    pub amount_value: Option<String>,
    pub price: Option<String>,
    pub in_stock: bool,
}

#[component]
pub fn ProductPage(product: ProductForm) -> impl IntoView {
    let ProductForm {
        id,
        title,
        image,
        variants,
        f,
    } = product;
    let heading = match id {
        Some(_) => format!("Upravit produkt: {title}"),
        None => "Nový produkt".to_string(),
    };
    view! {
        <a class="admin-back-link" href="/admin/catalog_products">"← Zpět na katalog"</a>
        <h1 class="admin-page-title">{heading}</h1>
        <Fields id=id image=image f=f/>
        {id.map(|id| view! { <Variants product_id=id variants=variants/> })}
    }
}

/// A field with its label and an optional hint under it.
pub(super) fn group(
    name: &'static str,
    label: &'static str,
    wide: bool,
    field: impl IntoView,
    hint: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="form-group" class:form-group-wide=wide>
            <label for=name>{label}</label>
            {field}
            {hint.map(|hint| view! { <p class="admin-form-hint">{hint}</p> })}
        </div>
    }
}

#[component]
fn Fields(id: Option<i64>, image: Option<String>, f: Form) -> impl IntoView {
    let action = match id {
        Some(id) => format!("/admin/catalog_products/{id}"),
        None => "/admin/catalog_products".to_string(),
    };
    let text = |name, label, wide| group(name, label, wide, text_field(&f, name), None);
    view! {
        <form class="admin-form" enctype="multipart/form-data" action=action accept-charset="UTF-8" method="post">
            <CsrfField/>
            <ErrorsBox f=f.clone()/>
            <div class="admin-form-grid">
                {group(
                    "key",
                    "Klíč (interní identifikátor)",
                    false,
                    text_field(&f, "key"),
                    Some(
                        "Bez diakritiky a mezer, např. \"smrk\", \"tramy\". Používá se v adrese produktu (/eshop/klic) - po vytvoření produktu ho už neměňte.",
                    ),
                )}
                {group(
                    "template",
                    "Šablona zobrazení",
                    false,
                    select(
                        &f,
                        "template",
                        &[
                            ("Palivové dřevo (kontejner + bedny)", Some("firewood")),
                            ("Řezivo (jedna matice rozměr × délka)", Some("lumber")),
                            ("Jednoduché varianty (bez matice)", Some("simple_variant")),
                        ],
                    ),
                    None,
                )}
                {group(
                    "category",
                    "Kategorie filtru",
                    false,
                    select(
                        &f,
                        "category",
                        &[
                            ("Palivové dřevo", Some("palivove")),
                            ("Stavební řezivo", Some("rezivo")),
                            ("Výrobní zbytky", Some("zbytky")),
                            ("Okrasné kamenivo", Some("kamenivo")),
                        ],
                    ),
                    None,
                )}
                {group(
                    "hardness",
                    "Tvrdost dřeva (jen pro palivové dřevo)",
                    false,
                    select(&f, "hardness", &[("— nepoužito —", None), ("Tvrdé", Some("hard")), ("Měkké", Some("soft"))]),
                    None,
                )}
                <div class="form-group">
                    <label for="image">"Obrázek"</label>
                    {image.map(|src| view! { <img class="admin-image-preview" src=src/> })}
                    {file_field(&f, "image", "image/*")}
                    <p class="admin-form-hint">
                        "Nahrajte novou fotku pro nahrazení aktuální. Necháte-li prázdné, zůstane aktuální obrázek."
                    </p>
                </div>
                {group("position", "Pořadí v katalogu", false, number_field(&f, "position"), None)}
            </div>
            <label class="custom-checkbox-label checkout-checkbox">
                {check_box(&f, "active")}
                <span class="custom-checkbox"></span>
                <span>"Aktivní (zobrazuje se v e-shopu)"</span>
            </label>
            <h2 class="admin-section-title">"Český obsah"</h2>
            <div class="admin-form-grid">
                {text("title", "Název", false)}
                {group(
                    "type_label",
                    "Popisek (typ dřeva / konstantní popis)",
                    false,
                    text_field(&f, "type_label"),
                    Some(
                        "Pro palivové dřevo: \"Tvrdé palivové dřevo\" apod. Pro řezivo/zbytky: krátký popis zobrazený u každé varianty.",
                    ),
                )}
                {text("subtitle", "Podtitulek (jen palivové dřevo)", true)}
                {group("description", "Popis", true, text_area(&f, "description"), None)}
                {text("drying_note", "Poznámka k sušení (jen palivové dřevo)", false)}
                {text("image_alt", "Popisek obrázku (alt text)", false)}
            </div>
            <h2 class="admin-section-title">"Německý obsah"</h2>
            <p class="admin-form-hint" style="margin-bottom: 14px">"Pokud necháte prázdné, zobrazí se automaticky český text."</p>
            <div class="admin-form-grid">
                {text("title_de", "Název (DE)", false)}
                {text("type_label_de", "Popisek (DE)", false)}
                {text("subtitle_de", "Podtitulek (DE)", true)}
                {group("description_de", "Popis (DE)", true, text_area(&f, "description_de"), None)}
                {text("drying_note_de", "Poznámka k sušení (DE)", false)}
                {text("image_alt_de", "Popisek obrázku (DE)", false)}
            </div>
            <div class="admin-form-actions">
                <Submit label="Uložit produkt" class="btn-primary"/>
                <a class="btn-outline-dark" href="/admin/catalog_products">"Zpět na katalog"</a>
            </div>
        </form>
    }
}

#[component]
fn Variants(product_id: i64, variants: Vec<VariantRow>) -> impl IntoView {
    let list = if variants.is_empty() {
        view! {
            <div class="admin-empty-state">
                "Tento produkt zatím nemá žádné varianty - bez alespoň jedné varianty se nedá koupit."
            </div>
        }
        .into_any()
    } else {
        view! {
            <table class="admin-table">
                <thead>
                    <tr>
                        <th>"Obrázek"</th>
                        <th>"Popis"</th>
                        <th>"Délka"</th>
                        <th>"Množství"</th>
                        <th>"Cena (Kč)"</th>
                        <th>"Skladem"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {variants
                        .into_iter()
                        .map(|variant| {
                            let url = format!("/admin/catalog_products/{product_id}/catalog_variants/{}", variant.id);
                            view! {
                                <tr>
                                    <td>
                                        {match variant.image {
                                            Some(src) => view! { <img class="admin-table-thumb" src=src/> }.into_any(),
                                            None => view! { <span style="color:#999; font-size:0.8rem">"produkt"</span> }.into_any(),
                                        }}
                                    </td>
                                    <td>
                                        {variant.description}
                                        {variant
                                            .group
                                            .map(|group| {
                                                view! {
                                                    <br/>
                                                    <span style="color:#999; font-size:0.8rem">{group}</span>
                                                }
                                            })}
                                    </td>
                                    <td>{variant.length_label}</td>
                                    <td>{variant.amount_value}</td>
                                    <td>{variant.price}</td>
                                    <td>{yes_no(variant.in_stock)}</td>
                                    <td>
                                        <a href=format!("{url}/edit")>"Upravit"</a>
                                        " "
                                        <ActionButton
                                            label="Smazat"
                                            action=format!("{url}/delete")
                                            class="admin-danger-btn"
                                            style="margin-left: 8px"
                                            confirm="Opravdu smazat tuto variantu?"
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
        <h2 class="admin-section-title">"Varianty a ceny"</h2>
        <div class="admin-toolbar">
            <p class="admin-form-hint" style="margin: 0">
                "Každá varianta je jedna konkrétní kombinace, kterou si zákazník může koupit (rozměr, délka, množství...) se svojí vlastní cenou."
            </p>
            <a class="btn-primary" href=format!("/admin/catalog_products/{product_id}/catalog_variants/new")>"+ Nová varianta"</a>
        </div>
        {list}
    }
}
