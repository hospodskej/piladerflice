//! Admin: the catalogue (products and their variants with prices).

use super::form::{values_for, Errors, Form, Values};
use super::{czk, AdminLayout, ConfirmButton};
use crate::pages::loaded;
use crate::visitor::visitor;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

const TEMPLATES: [(&str, &str); 3] = [
    ("Palivové dřevo (kontejner + bedny)", "firewood"),
    ("Řezivo (jedna matice rozměr × délka)", "lumber"),
    ("Jednoduché varianty (bez matice)", "simple_variant"),
];
const CATEGORIES: [(&str, &str); 4] = [("Palivové dřevo", "palivove"), ("Stavební řezivo", "rezivo"), ("Výrobní zbytky", "zbytky"), ("Okrasné kamenivo", "kamenivo")];
const HARDNESS: [(&str, &str); 3] = [("— nepoužito —", ""), ("Tvrdé", "hard"), ("Měkké", "soft")];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductRow {
    id: i64,
    key: String,
    title: String,
    template: String,
    category: String,
    variants: i64,
    starting_price: Option<i64>,
    active: bool,
}

#[server(endpoint = "admin/catalog")]
pub async fn load_products() -> Result<Vec<ProductRow>, ServerFnError> {
    super::require_admin()?;
    let db = &expect_context::<crate::state::AppState>().db;
    Ok(sqlx::query!(
        r#"SELECT p.id AS "id!", p.key, p.title, p.template, p.category, p.active,
                  (SELECT COUNT(*) FROM catalog_variants v WHERE v.catalog_product_id = p.id) AS "variants!: i64",
                  (SELECT COALESCE(MIN(CASE WHEN v.in_stock THEN v.price_czk END), MIN(v.price_czk)) FROM catalog_variants v WHERE v.catalog_product_id = p.id) AS "starting_price: i64"
           FROM catalog_products p ORDER BY p.position, p.id"#
    )
    .fetch_all(db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .into_iter()
    .map(|p| ProductRow {
        id: p.id,
        key: p.key,
        title: p.title,
        template: p.template,
        category: p.category,
        variants: p.variants,
        starting_price: p.starting_price,
        active: p.active != 0,
    })
    .collect())
}

#[component]
pub fn ProductsPage() -> impl IntoView {
    view! {
        <AdminLayout>
            <div class="admin-toolbar">
                <h1 class="admin-page-title">"Katalog a ceny"</h1>
                <a href="/admin/catalog_products/new" class="btn-primary">"+ Nový produkt"</a>
            </div>
            {loaded(load_products(), |rows: Vec<ProductRow>| if rows.is_empty() {
                view! { <div class="admin-empty-state">"Zatím žádné produkty v katalogu."</div> }.into_any()
            } else {
                view! {
                    <table class="admin-table">
                        <thead><tr><th>"Produkt"</th><th>"Šablona"</th><th>"Kategorie"</th><th>"Varianty"</th><th>"Od ceny"</th><th>"Aktivní"</th><th></th></tr></thead>
                        <tbody>
                            {rows.into_iter().map(|p| view! {
                                <tr>
                                    <td><strong>{p.title}</strong><br/><span style="color:#999; font-size:0.8rem;">{p.key}</span></td>
                                    <td>{p.template}</td>
                                    <td>{p.category}</td>
                                    <td>{p.variants}</td>
                                    <td>{match p.starting_price {
                                        Some(price) => czk(price).into_any(),
                                        None => view! { <span style="color:#D33;">"bez ceny"</span> }.into_any(),
                                    }}</td>
                                    <td>{if p.active { "Ano" } else { "Ne" }}</td>
                                    <td><a href=format!("/admin/catalog_products/{}/edit", p.id) class="admin-edit-btn">"Upravit"</a></td>
                                </tr>
                            }).collect_view()}
                        </tbody>
                    </table>
                }.into_any()
            })}
        </AdminLayout>
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VariantRow {
    id: i64,
    image: Option<String>,
    description: String,
    group: Option<String>,
    length: String,
    amount: String,
    price: i64,
    in_stock: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProductForm {
    id: Option<i64>,
    values: Values,
    image: Option<String>,
    variants: Vec<VariantRow>,
}

/// A product (or a blank one for `None`) with its variants.
#[server(endpoint = "admin/catalog-product")]
pub async fn load_product(id: Option<i64>) -> Result<Option<ProductForm>, ServerFnError> {
    super::require_admin()?;
    let Some(id) = id else {
        let values = [("template", "firewood"), ("category", "palivove"), ("active", "1"), ("position", "0")]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        return Ok(Some(ProductForm { values, ..ProductForm::default() }));
    };
    let db = &expect_context::<crate::state::AppState>().db;
    let err = |e: sqlx::Error| ServerFnError::new(e.to_string());
    let Some(p) = sqlx::query!("SELECT * FROM catalog_products WHERE id = ?", id).fetch_optional(db).await.map_err(err)? else {
        return Ok(None);
    };
    let o = |s: Option<String>| s.unwrap_or_default();
    let values: Values = [
        ("key", p.key),
        ("template", p.template.clone()),
        ("category", p.category),
        ("hardness", o(p.hardness)),
        ("position", p.position.to_string()),
        ("active", p.active.to_string()),
        ("title", p.title),
        ("title_de", o(p.title_de)),
        ("type_label", o(p.type_label)),
        ("type_label_de", o(p.type_label_de)),
        ("subtitle", o(p.subtitle)),
        ("subtitle_de", o(p.subtitle_de)),
        ("description", o(p.description)),
        ("description_de", o(p.description_de)),
        ("drying_note", o(p.drying_note)),
        ("drying_note_de", o(p.drying_note_de)),
        ("image_alt", o(p.image_alt)),
        ("image_alt_de", o(p.image_alt_de)),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v))
    .collect();
    let lumber = p.template == "lumber";
    let variants = sqlx::query!("SELECT * FROM catalog_variants WHERE catalog_product_id = ? ORDER BY position, id", id)
        .fetch_all(db)
        .await
        .map_err(err)?
        .into_iter()
        .map(|v| VariantRow {
            id: v.id,
            image: v.image,
            description: if lumber {
                [v.grade.clone().filter(|g| !g.is_empty()), v.width_mm.zip(v.height_mm).map(|(w, h)| format!("{w}×{h} mm"))]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" / ")
            } else {
                o(v.variant_label)
            },
            group: v.variant_group.as_deref().map(|g| match (g, v.stacked != 0) {
                ("bedny", _) => "Skládané (bedny)",
                (_, true) => "Skládané (kontejner)",
                _ => "Sypané (kontejner)",
            }.to_owned()),
            length: o(v.length_label),
            amount: v.amount_value.map(|a| a.to_string()).unwrap_or_default(),
            price: v.price_czk,
            in_stock: v.in_stock != 0,
        })
        .collect();
    Ok(Some(ProductForm { id: Some(id), values, image: p.image, variants }))
}

fn product_form(f: &ProductForm, values: &Values, errors: Vec<String>) -> impl IntoView + use<> {
    let form = Form { model: "catalog_product", values };
    let action = f.id.map_or_else(|| "/admin/catalog_products".to_owned(), |id| format!("/admin/catalog_products/{id}"));
    view! {
        <form action=action method="post" enctype="multipart/form-data" class="admin-form">
            <Errors errors=errors/>
            <div class="admin-form-grid">
                {form.text("key", "Klíč (interní identifikátor)", Some("Bez diakritiky a mezer, např. \"smrk\", \"tramy\". Používá se v adrese produktu (/eshop/klic) - po vytvoření produktu ho už neměňte."), false)}
                {form.select("template", "Šablona zobrazení", &TEMPLATES, None)}
                {form.select("category", "Kategorie filtru", &CATEGORIES, None)}
                {form.select("hardness", "Tvrdost dřeva (jen pro palivové dřevo)", &HARDNESS, None)}
                {form.image("image", "Obrázek", f.image.clone(), "Nahrajte novou fotku pro nahrazení aktuální. Necháte-li prázdné, zůstane aktuální obrázek.")}
                {form.number("position", "Pořadí v katalogu", None)}
            </div>
            {form.checkbox("active", "Aktivní (zobrazuje se v e-shopu)")}
            <h2 class="admin-section-title">"Český obsah"</h2>
            <div class="admin-form-grid">
                {form.text("title", "Název", None, false)}
                {form.text("type_label", "Popisek (typ dřeva / konstantní popis)", Some("Pro palivové dřevo: \"Tvrdé palivové dřevo\" apod. Pro řezivo/zbytky: krátký popis zobrazený u každé varianty."), false)}
                {form.text("subtitle", "Podtitulek (jen palivové dřevo)", None, true)}
                {form.textarea("description", "Popis", true)}
                {form.text("drying_note", "Poznámka k sušení (jen palivové dřevo)", None, false)}
                {form.text("image_alt", "Popisek obrázku (alt text)", None, false)}
            </div>
            <h2 class="admin-section-title">"Německý obsah"</h2>
            <p class="admin-form-hint" style="margin-bottom: 14px;">"Pokud necháte prázdné, zobrazí se automaticky český text."</p>
            <div class="admin-form-grid">
                {form.text("title_de", "Název (DE)", None, false)}
                {form.text("type_label_de", "Popisek (DE)", None, false)}
                {form.text("subtitle_de", "Podtitulek (DE)", None, true)}
                {form.textarea("description_de", "Popis (DE)", true)}
                {form.text("drying_note_de", "Poznámka k sušení (DE)", None, false)}
                {form.text("image_alt_de", "Popisek obrázku (DE)", None, false)}
            </div>
            <div class="admin-form-actions">
                <input type="submit" value="Uložit produkt" class="btn-primary"/>
                <a href="/admin/catalog_products" class="btn-outline-dark">"Zpět na katalog"</a>
            </div>
        </form>
    }
}

#[component]
pub fn ProductPage() -> impl IntoView {
    let id: Option<i64> = use_params_map().with_untracked(|p| p.get("id").and_then(|id| id.parse().ok()));
    view! {
        <AdminLayout>
            <a href="/admin/catalog_products" class="admin-back-link">"← Zpět na katalog"</a>
            {loaded(load_product(id), move |f: Option<ProductForm>| {
                let Some(f) = f else { return view! { <p>"Produkt nenalezen."</p> }.into_any() };
                let v = visitor();
                let (values, errors) = values_for(&v, f.values.clone());
                let title = f.values.get("title").cloned().unwrap_or_default();
                match f.id {
                    None => view! {
                        <h1 class="admin-page-title">"Nový produkt"</h1>
                        {product_form(&f, &values, errors)}
                    }.into_any(),
                    Some(id) => view! {
                        <h1 class="admin-page-title">"Upravit produkt: "{title}</h1>
                        {product_form(&f, &values, errors)}
                        <h2 class="admin-section-title">"Varianty a ceny"</h2>
                        <div class="admin-toolbar">
                            <p class="admin-form-hint" style="margin: 0;">"Každá varianta je jedna konkrétní kombinace, kterou si zákazník může koupit (rozměr, délka, množství...) se svojí vlastní cenou."</p>
                            <a href=format!("/admin/catalog_products/{id}/catalog_variants/new") class="btn-primary">"+ Nová varianta"</a>
                        </div>
                        {if f.variants.is_empty() {
                            view! { <div class="admin-empty-state">"Tento produkt zatím nemá žádné varianty - bez alespoň jedné varianty se nedá koupit."</div> }.into_any()
                        } else {
                            view! {
                                <table class="admin-table">
                                    <thead><tr><th>"Obrázek"</th><th>"Popis"</th><th>"Délka"</th><th>"Množství"</th><th>"Cena (Kč)"</th><th>"Skladem"</th><th></th></tr></thead>
                                    <tbody>
                                        {f.variants.into_iter().map(|x| view! {
                                            <tr>
                                                <td>{match x.image {
                                                    Some(src) => view! { <img src=src class="admin-table-thumb"/> }.into_any(),
                                                    None => view! { <span style="color:#999; font-size:0.8rem;">"produkt"</span> }.into_any(),
                                                }}</td>
                                                <td>{x.description}{x.group.map(|g| view! { <br/><span style="color:#999; font-size:0.8rem;">{g}</span> })}</td>
                                                <td>{x.length}</td>
                                                <td>{x.amount}</td>
                                                <td>{czk(x.price)}</td>
                                                <td>{if x.in_stock { "Ano" } else { "Ne" }}</td>
                                                <td>
                                                    <div class="admin-row-actions">
                                                        <a href=format!("/admin/catalog_products/{id}/catalog_variants/{}/edit", x.id) class="admin-edit-btn">"Upravit"</a>
                                                        <ConfirmButton action=format!("/admin/catalog_products/{id}/catalog_variants/{}/smazat", x.id) text="Smazat" confirm="Opravdu smazat tuto variantu?"/>
                                                    </div>
                                                </td>
                                            </tr>
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }.into_any()
                        }}
                    }.into_any(),
                }
            })}
        </AdminLayout>
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VariantForm {
    product_id: i64,
    product_title: String,
    template: String,
    /// "včetně DPH" or "bez DPH", and the hint about the customer price.
    vat_note: (bool, u8, i64),
    id: Option<i64>,
    values: Values,
    image: Option<String>,
}

#[server(endpoint = "admin/catalog-variant")]
pub async fn load_variant(product_id: i64, id: Option<i64>) -> Result<Option<VariantForm>, ServerFnError> {
    let v = super::require_admin()?;
    let db = &expect_context::<crate::state::AppState>().db;
    let err = |e: sqlx::Error| ServerFnError::new(e.to_string());
    let Some(p) = sqlx::query!("SELECT title, template, category FROM catalog_products WHERE id = ?", product_id).fetch_optional(db).await.map_err(err)? else {
        return Ok(None);
    };
    let rule = v.shop.vat.get(p.category.parse().unwrap_or(domain::Category::Firewood));
    let vat_note = (rule.prices_include_vat, rule.rate_cz, rule.customer_price(domain::Czk(1000)).get());
    let mut form = VariantForm { product_id, product_title: p.title, template: p.template, vat_note, id, ..VariantForm::default() };
    match id {
        None => {
            form.values = [("variant_group", "kontejner"), ("in_stock", "1"), ("position", "0")].into_iter().map(|(k, v)| (k.to_owned(), v.to_owned())).collect();
        }
        Some(id) => {
            let Some(x) = sqlx::query!("SELECT * FROM catalog_variants WHERE id = ? AND catalog_product_id = ?", id, product_id).fetch_optional(db).await.map_err(err)? else {
                return Ok(None);
            };
            let o = |s: Option<String>| s.unwrap_or_default();
            let n = |s: Option<i64>| s.map(|n| n.to_string()).unwrap_or_default();
            form.values = [
                ("key", x.key),
                ("variant_group", o(x.variant_group)),
                ("length_label", o(x.length_label)),
                ("variant_label", o(x.variant_label)),
                ("variant_label_de", o(x.variant_label_de)),
                ("amount_value", n(x.amount_value)),
                ("stacked", x.stacked.to_string()),
                ("price_czk", x.price_czk.to_string()),
                ("in_stock", x.in_stock.to_string()),
                ("position", x.position.to_string()),
                ("grade", o(x.grade)),
                ("grade_de", o(x.grade_de)),
                ("width_mm", n(x.width_mm)),
                ("height_mm", n(x.height_mm)),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect();
            form.image = x.image;
        }
    }
    Ok(Some(form))
}

#[component]
pub fn VariantPage() -> impl IntoView {
    let (product_id, id) = use_params_map().with_untracked(|p| {
        (p.get("product_id").and_then(|i| i.parse().ok()).unwrap_or(0), p.get("id").and_then(|i| i.parse::<i64>().ok()))
    });
    view! {
        <AdminLayout>
            <a href=format!("/admin/catalog_products/{product_id}/edit") class="admin-back-link">"← Zpět na produkt"</a>
            {loaded(load_variant(product_id, id), move |f: Option<VariantForm>| {
                let Some(f) = f else { return view! { <p>"Varianta nenalezena."</p> }.into_any() };
                let v = visitor();
                let (values, errors) = values_for(&v, f.values.clone());
                let form = Form { model: "catalog_variant", values: &values };
                let action = f.id.map_or_else(|| format!("/admin/catalog_products/{product_id}/catalog_variants"), |id| format!("/admin/catalog_products/{product_id}/catalog_variants/{id}"));
                let (includes_vat, rate, example) = f.vat_note;
                let firewood = f.template == "firewood";
                let lumber = f.template == "lumber";
                view! {
                    <h1 class="admin-page-title">{if f.id.is_some() { "Upravit variantu: " } else { "Nová varianta: " }}{f.product_title.clone()}</h1>
                    <form action=action method="post" enctype="multipart/form-data" class="admin-form">
                        <Errors errors=errors/>
                        <div class="admin-form-grid">
                            {form.image("image", "Obrázek varianty", f.image.clone(), "Volitelné. Nahrajte obrázek pro tuto konkrétní variantu (např. jiné množství nebo rozměr), pokud se má lišit od výchozího obrázku produktu. Necháte-li prázdné, zobrazí se obrázek produktu.")}
                            {form.text("key", "Klíč (interní identifikátor)", Some("Bez diakritiky a mezer, jen pro rozlišení mezi variantami tohoto produktu, např. \"100cm-15prm\"."), false)}
                            <div class="form-group">
                                <label for="catalog_variant_price_czk">{format!("Cena (Kč, {})", if includes_vat { "včetně DPH" } else { "bez DPH" })}</label>
                                <input type="number" name="catalog_variant[price_czk]" id="catalog_variant_price_czk" value=form.value("price_czk")/>
                                <p class="admin-form-hint">
                                    {(!includes_vat).then(|| format!("Zákazník uvidí cenu s {rate} % DPH (např. 1 000 Kč → {}). ", czk(example)))}
                                    "Jak se zadávají ceny a sazby DPH pro Česko a Rakousko, se nastavuje v "<a href="/admin/dph">"DPH"</a>". "
                                    "Cena v eurech se dopočítává automaticky podle aktuálního kurzu."
                                </p>
                            </div>
                            {firewood.then(|| view! {
                                {form.select("variant_group", "Dodání v", &[("Kontejneru", "kontejner"), ("Bednách", "bedny")], None)}
                                <div class="form-group form-group-wide">
                                    {form.checkbox("stacked", "Dřevo je skládané (rovnané)")}
                                    <p class="admin-form-hint">"Zaškrtněte u dřeva v kontejneru, které je poskládané (např. 50 cm a 1 m). Nezaškrtnuté = sypané (např. 25 a 33 cm). Dřevo v bednách je vždy skládané."</p>
                                </div>
                                {form.number("amount_value", "Množství (PRM)", None)}
                            })}
                            {(firewood || lumber).then(|| form.input("text", "length_label", "Délka", None, false, Some("100 cm nebo 3000 mm")))}
                            {if lumber {
                                view! {
                                    {form.number("width_mm", "Šířka (mm)", Some("Nechte prázdné, pokud produkt šířku nerozlišuje (např. prkna)."))}
                                    {form.number("height_mm", "Výška (mm)", Some("Nechte prázdné, pokud produkt výšku nerozlišuje."))}
                                    {form.text("grade", "Třída (CZ)", Some("Nechte prázdné, pokud produkt třídu nerozlišuje."), false)}
                                    {form.text("grade_de", "Třída (DE)", Some("Pokud necháte prázdné, zobrazí se automaticky český text."), false)}
                                }.into_any()
                            } else {
                                view! {
                                    {form.text("variant_label", "Popis varianty (CZ)", None, true)}
                                    {form.text("variant_label_de", "Popis varianty (DE)", Some("Pokud necháte prázdné, zobrazí se automaticky český text."), true)}
                                }.into_any()
                            }}
                            {form.number("position", "Pořadí", None)}
                        </div>
                        {form.checkbox("in_stock", "Skladem")}
                        <div class="admin-form-actions">
                            <input type="submit" value="Uložit variantu" class="btn-primary"/>
                            <a href=format!("/admin/catalog_products/{product_id}/edit") class="btn-outline-dark">"Zpět na produkt"</a>
                        </div>
                    </form>
                }.into_any()
            })}
        </AdminLayout>
    }
}

#[cfg(feature = "ssr")]
pub mod handlers {
    use super::super::form::parse::{checked, fields, int, multipart, text};
    use super::super::handlers::{done, guard, refused, save_upload};
    use crate::state::AppState;
    use crate::visitor::Visitor;
    use axum::extract::{Multipart, Path, State};
    use axum::response::Response;
    use axum::Extension;
    use tower_sessions::Session;

    fn is_key(key: &str) -> bool {
        !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    }

    async fn save_product(state: &AppState, session: &Session, id: Option<i64>, form: Multipart) -> Response {
        let (pairs, files) = multipart(form).await;
        let values = fields(&pairs, "catalog_product");
        let form_path = id.map_or_else(|| "/admin/catalog_products/new".to_owned(), |id| format!("/admin/catalog_products/{id}/edit"));
        let mut errors = Vec::new();
        let key = text(&values, "key").unwrap_or_default();
        if key.is_empty() {
            errors.push("Klíč je povinné pole".to_owned());
        } else {
            let taken = sqlx::query_scalar!("SELECT id FROM catalog_products WHERE key = ?", key).fetch_optional(&state.db).await.ok().flatten();
            if taken.is_some_and(|other| Some(other) != id) {
                errors.push("Klíč už je použitý".to_owned());
            }
        }
        let template = text(&values, "template").filter(|t| ["firewood", "lumber", "simple_variant"].contains(&t.as_str()));
        if template.is_none() {
            errors.push("Šablona zobrazení má neplatnou hodnotu".to_owned());
        }
        let category = text(&values, "category").filter(|c| domain::Category::ALL.iter().any(|x| x.as_str() == c));
        if category.is_none() {
            errors.push("Kategorie filtru má neplatnou hodnotu".to_owned());
        }
        let hardness = text(&values, "hardness");
        if hardness.as_deref().is_some_and(|h| h != "hard" && h != "soft") {
            errors.push("Tvrdost dřeva má neplatnou hodnotu".to_owned());
        }
        let title = text(&values, "title").unwrap_or_default();
        let position = int(&values, "position").unwrap_or(None).unwrap_or(0);
        let mut image = None;
        if let Some((_, bytes)) = files.iter().find(|(name, _)| name == "catalog_product[image]") {
            match save_upload(state, bytes).await {
                Ok(path) => image = Some(path),
                Err(problem) => errors.push(problem),
            }
        }
        if !errors.is_empty() {
            return refused(session, &form_path, values, errors).await;
        }
        let t = |f: &str| text(&values, f);
        let active = i64::from(checked(&values, "active"));
        let (template, category) = (template.unwrap_or_default(), category.unwrap_or_default());
        let (title_de, type_label, type_label_de, subtitle, subtitle_de) = (t("title_de"), t("type_label"), t("type_label_de"), t("subtitle"), t("subtitle_de"));
        let (description, description_de, drying_note, drying_note_de, image_alt, image_alt_de) =
            (t("description"), t("description_de"), t("drying_note"), t("drying_note_de"), t("image_alt"), t("image_alt_de"));
        let saved = match id {
            Some(id) => sqlx::query!(
                "UPDATE catalog_products SET key = ?, template = ?, category = ?, hardness = ?, active = ?, position = ?, title = ?, title_de = ?,
                        type_label = ?, type_label_de = ?, subtitle = ?, subtitle_de = ?, description = ?, description_de = ?,
                        drying_note = ?, drying_note_de = ?, image_alt = ?, image_alt_de = ?, image = COALESCE(?, image),
                        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?",
                key, template, category, hardness, active, position, title, title_de, type_label, type_label_de, subtitle, subtitle_de,
                description, description_de, drying_note, drying_note_de, image_alt, image_alt_de, image, id
            )
            .execute(&state.db)
            .await
            .map(|_| id),
            None => sqlx::query_scalar!(
                r#"INSERT INTO catalog_products (key, template, category, hardness, active, position, title, title_de, type_label, type_label_de,
                        subtitle, subtitle_de, description, description_de, drying_note, drying_note_de, image_alt, image_alt_de, image)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id AS "id!""#,
                key, template, category, hardness, active, position, title, title_de, type_label, type_label_de, subtitle, subtitle_de,
                description, description_de, drying_note, drying_note_de, image_alt, image_alt_de, image
            )
            .fetch_one(&state.db)
            .await,
        };
        match saved {
            Ok(saved_id) => {
                let notice = if id.is_some() { "Produkt byl uložen." } else { "Produkt byl vytvořen. Nyní k němu přidejte varianty s cenami." };
                done(session, &format!("/admin/catalog_products/{saved_id}/edit"), notice).await
            }
            Err(e) => refused(session, &form_path, values, vec![e.to_string()]).await,
        }
    }

    pub async fn create_product(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, form: Multipart) -> Response {
        if let Err(r) = guard(&session, &v).await {
            return r;
        }
        save_product(&state, &session, None, form).await
    }

    pub async fn update_product(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Path(id): Path<i64>, form: Multipart) -> Response {
        if let Err(r) = guard(&session, &v).await {
            return r;
        }
        save_product(&state, &session, Some(id), form).await
    }

    pub async fn delete_product(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Path(id): Path<i64>) -> Response {
        if let Err(r) = guard(&session, &v).await {
            return r;
        }
        let _ = sqlx::query!("DELETE FROM catalog_products WHERE id = ?", id).execute(&state.db).await;
        done(&session, "/admin/catalog_products", "Produkt byl smazán.").await
    }

    async fn save_variant(state: &AppState, session: &Session, product_id: i64, id: Option<i64>, form: Multipart) -> Response {
        let (pairs, files) = multipart(form).await;
        let values = fields(&pairs, "catalog_variant");
        let form_path = id.map_or_else(
            || format!("/admin/catalog_products/{product_id}/catalog_variants/new"),
            |id| format!("/admin/catalog_products/{product_id}/catalog_variants/{id}/edit"),
        );
        let mut errors = Vec::new();
        let key = text(&values, "key").unwrap_or_default();
        if key.is_empty() {
            errors.push("Klíč je povinné pole".to_owned());
        } else if !is_key(&key) || sqlx::query_scalar!("SELECT id FROM catalog_variants WHERE catalog_product_id = ? AND key = ?", product_id, key)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
            .is_some_and(|other| Some(other) != id)
        {
            errors.push("Klíč už je použitý".to_owned());
        }
        let price = match int(&values, "price_czk") {
            Ok(Some(p)) if p > 0 => Some(p),
            Ok(Some(_)) => {
                errors.push("Cena musí být větší než 0".to_owned());
                None
            }
            Ok(None) | Err(()) => {
                errors.push("Cena musí být číslo".to_owned());
                None
            }
        };
        let group = text(&values, "variant_group");
        if group.as_deref().is_some_and(|g| g != "kontejner" && g != "bedny") {
            errors.push("Dodání v má neplatnou hodnotu".to_owned());
        }
        let number = |field: &str, label: &str, errors: &mut Vec<String>| match int(&values, field) {
            Ok(n) => n,
            Err(()) => {
                errors.push(format!("{label} musí být číslo"));
                None
            }
        };
        let amount = number("amount_value", "Množství", &mut errors);
        let width = number("width_mm", "Šířka", &mut errors);
        let height = number("height_mm", "Výška", &mut errors);
        let position = number("position", "Pořadí", &mut errors).unwrap_or(0);
        let mut image = None;
        if let Some((_, bytes)) = files.iter().find(|(name, _)| name == "catalog_variant[image]") {
            match save_upload(state, bytes).await {
                Ok(path) => image = Some(path),
                Err(problem) => errors.push(problem),
            }
        }
        if !errors.is_empty() {
            return refused(session, &form_path, values, errors).await;
        }
        let t = |f: &str| text(&values, f);
        let (stacked, in_stock) = (i64::from(checked(&values, "stacked")), i64::from(checked(&values, "in_stock")));
        let price = price.unwrap_or(1);
        let (length, label, label_de, grade, grade_de) = (t("length_label"), t("variant_label"), t("variant_label_de"), t("grade"), t("grade_de"));
        let saved = match id {
            Some(id) => sqlx::query!(
                "UPDATE catalog_variants SET key = ?, variant_group = ?, length_label = ?, variant_label = ?, variant_label_de = ?, amount_value = ?,
                        stacked = ?, price_czk = ?, in_stock = ?, position = ?, grade = ?, grade_de = ?, width_mm = ?, height_mm = ?,
                        image = COALESCE(?, image), updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                 WHERE id = ? AND catalog_product_id = ?",
                key, group, length, label, label_de, amount, stacked, price, in_stock, position, grade, grade_de, width, height, image, id, product_id
            )
            .execute(&state.db)
            .await
            .map(|_| ()),
            None => sqlx::query!(
                "INSERT INTO catalog_variants (catalog_product_id, key, variant_group, length_label, variant_label, variant_label_de, amount_value,
                        stacked, price_czk, in_stock, position, grade, grade_de, width_mm, height_mm, image)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                product_id, key, group, length, label, label_de, amount, stacked, price, in_stock, position, grade, grade_de, width, height, image
            )
            .execute(&state.db)
            .await
            .map(|_| ()),
        };
        match saved {
            Ok(()) => done(session, &format!("/admin/catalog_products/{product_id}/edit"), if id.is_some() { "Varianta byla uložena." } else { "Varianta byla přidána." }).await,
            Err(e) => refused(session, &form_path, values, vec![e.to_string()]).await,
        }
    }

    pub async fn create_variant(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Path(product_id): Path<i64>, form: Multipart) -> Response {
        if let Err(r) = guard(&session, &v).await {
            return r;
        }
        save_variant(&state, &session, product_id, None, form).await
    }

    pub async fn update_variant(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Path((product_id, id)): Path<(i64, i64)>, form: Multipart) -> Response {
        if let Err(r) = guard(&session, &v).await {
            return r;
        }
        save_variant(&state, &session, product_id, Some(id), form).await
    }

    pub async fn delete_variant(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Path((product_id, id)): Path<(i64, i64)>) -> Response {
        if let Err(r) = guard(&session, &v).await {
            return r;
        }
        let _ = sqlx::query!("DELETE FROM catalog_variants WHERE id = ? AND catalog_product_id = ?", id, product_id).execute(&state.db).await;
        done(&session, &format!("/admin/catalog_products/{product_id}/edit"), "Varianta byla smazána.").await
    }
}
