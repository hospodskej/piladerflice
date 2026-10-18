//! A product's variant form (/admin/catalog_products/{id}/catalog_variants):
//! the fields that the product's layout uses (container or crates for
//! firewood, sizes and grade for lumber, a label for the rest).

use leptos::prelude::*;

use super::form::{ErrorsBox, Form, Submit, check_box, file_field, number_field, select, text_field, text_field_with};
use super::products::group;
use crate::ui::CsrfField;

pub struct VariantForm {
    pub id: Option<i64>,
    pub product_id: i64,
    pub product_title: String,
    /// The product's layout: "firewood", "lumber" or "simple_variant".
    pub template: String,
    pub image: Option<String>,
    pub f: Form,
}

#[component]
pub fn VariantPage(variant: VariantForm) -> impl IntoView {
    let VariantForm {
        id,
        product_id,
        product_title,
        template,
        image,
        f,
    } = variant;
    let product = format!("/admin/catalog_products/{product_id}");
    let action = match id {
        Some(id) => format!("{product}/catalog_variants/{id}"),
        None => format!("{product}/catalog_variants"),
    };
    let heading = if id.is_some() {
        "Upravit variantu"
    } else {
        "Nová varianta"
    };
    let firewood = template == "firewood";
    let lumber = template == "lumber";
    let text = |name, label, wide, hint| group(name, label, wide, text_field(&f, name), hint);
    let number = |name, label, hint| group(name, label, false, number_field(&f, name), hint);
    let firewood_fields = firewood.then(|| {
        view! {
            {group("variant_group", "Dodání v", false, select(&f, "variant_group", &[("Kontejneru", Some("kontejner")), ("Bednách", Some("bedny"))]), None)}
            <div class="form-group form-group-wide">
                <label class="custom-checkbox-label checkout-checkbox">
                    {check_box(&f, "stacked")}
                    <span class="custom-checkbox"></span>
                    <span>"Dřevo je skládané (rovnané)"</span>
                </label>
                <p class="admin-form-hint">
                    "Zaškrtněte u dřeva v kontejneru, které je poskládané (např. 50 cm a 1 m). Nezaškrtnuté = sypané (např. 25 a 33 cm). Dřevo v bednách je vždy skládané."
                </p>
            </div>
            {number("amount_value", "Množství (PRM)", None)}
        }
    });
    let length = (firewood || lumber).then(|| {
        group(
            "length_label",
            "Délka",
            false,
            text_field_with(&f, "length_label", "100 cm nebo 3000 mm"),
            None,
        )
    });
    let rest = if lumber {
        view! {
            {number("width_mm", "Šířka (mm)", Some("Nechte prázdné, pokud produkt šířku nerozlišuje (např. prkna)."))}
            {number("height_mm", "Výška (mm)", Some("Nechte prázdné, pokud produkt výšku nerozlišuje."))}
            {text("grade", "Třída (CZ)", false, Some("Nechte prázdné, pokud produkt třídu nerozlišuje."))}
            {text("grade_de", "Třída (DE)", false, Some("Pokud necháte prázdné, zobrazí se automaticky český text."))}
        }
        .into_any()
    } else {
        view! {
            {text("variant_label", "Popis varianty (CZ)", true, None)}
            {text("variant_label_de", "Popis varianty (DE)", true, Some("Pokud necháte prázdné, zobrazí se automaticky český text."))}
        }
        .into_any()
    };
    view! {
        <a class="admin-back-link" href=format!("{product}/edit")>"← Zpět na produkt"</a>
        <h1 class="admin-page-title">{format!("{heading}: {product_title}")}</h1>
        <form class="admin-form" enctype="multipart/form-data" action=action accept-charset="UTF-8" method="post">
            <CsrfField/>
            <ErrorsBox f=f.clone()/>
            <div class="admin-form-grid">
                <div class="form-group">
                    <label for="image">"Obrázek varianty"</label>
                    {image.map(|src| view! { <img class="admin-image-preview" src=src/> })}
                    {file_field(&f, "image", "image/*")}
                    <p class="admin-form-hint">
                        "Volitelné. Nahrajte obrázek pro tuto konkrétní variantu (např. jiné množství nebo rozměr), pokud se má lišit od výchozího obrázku produktu. Necháte-li prázdné, zobrazí se obrázek produktu."
                    </p>
                </div>
                {text(
                    "key",
                    "Klíč (interní identifikátor)",
                    false,
                    Some("Bez diakritiky a mezer, jen pro rozlišení mezi variantami tohoto produktu, např. \"100cm-15prm\"."),
                )}
                {number("price_czk", "Cena (Kč)", Some("Cena v eurech se dopočítává automaticky podle aktuálního kurzu."))}
                {firewood_fields}
                {length}
                {rest}
                {number("position", "Pořadí", None)}
            </div>
            <label class="custom-checkbox-label checkout-checkbox">
                {check_box(&f, "in_stock")}
                <span class="custom-checkbox"></span>
                <span>"Skladem"</span>
            </label>
            <div class="admin-form-actions">
                <Submit label="Uložit variantu" class="btn-primary"/>
                <a class="btn-outline-dark" href=format!("{product}/edit")>"Zpět na produkt"</a>
            </div>
        </form>
    }
}
