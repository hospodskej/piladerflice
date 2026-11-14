//! Admin: the price list, the inquiry form's options and the promos.

use super::form::{values_for, Errors, Form, Values};
use super::{AdminLayout, ConfirmButton};
use crate::i18n::{t, Locale};
use crate::pages::loaded;
use crate::visitor::visitor;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

/// The price list's categories with their fields, in order.
pub const PRICELIST: [(&str, &[&str]); 6] = [
    ("stavebni", &["item_name", "details", "price"]),
    ("palivove_volne", &["subcategory", "item_name", "price"]),
    ("palivove_skladane", &["subcategory", "item_name", "price"]),
    ("zbytky", &["item_name", "price"]),
    ("kamenivo", &["price"]),
    ("sluzby", &["item_name", "price"]),
];

fn section_title(category: &str) -> &'static str {
    use t::kontakt as k;
    let l = Locale::Cs;
    match category {
        "stavebni" => k::cenik_stavebni_title(l),
        "palivove_volne" => k::cenik_palivove_volne_title(l),
        "palivove_skladane" => k::cenik_palivove_skladane_title(l),
        "zbytky" => k::cenik_zbytky_title(l),
        "kamenivo" => k::cenik_kamenivo_title(l),
        _ => k::cenik_sluzeb_title(l),
    }
}

fn field_label(category: &str, field: &str) -> &'static str {
    let firewood = category.starts_with("palivove");
    match (field, firewood, category) {
        ("subcategory", ..) => "Dřevina",
        ("item_name", true, _) => "Délka",
        ("price", true, _) => "Cena za m³",
        ("details", ..) => "Délky",
        ("item_name", _, "sluzby") => "Služba",
        ("item_name", ..) => "Položka",
        _ => "Cena",
    }
}

const CONVERTED_PRICE_HINT: &str = "Německou cenu můžete nechat prázdnou – německá verze pak českou cenu automaticky přepočítá na eura podle aktuálního kurzu. Krychlový metr napište jako m³.";

fn hint(category: &str) -> String {
    match category {
        "palivove_volne" | "palivove_skladane" => "„/ m³“ se k ceně doplní automaticky. Německá cena se zobrazí přesně tak, jak ji zadáte (v eurech, bez přepočtu). Pro „není k dispozici“ zadejte ---. Položky se stejnou dřevinou se zobrazí v jedné tabulce.".into(),
        "kamenivo" => format!("Zobrazuje se jedna cena. {CONVERTED_PRICE_HINT}"),
        _ => CONVERTED_PRICE_HINT.into(),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PriceItem {
    id: i64,
    category: String,
    delivery_rate: bool,
    values: Values,
}

#[server(endpoint = "admin/pricelist")]
pub async fn load_pricelist() -> Result<(Vec<PriceItem>, i64), ServerFnError> {
    let v = super::require_admin()?;
    let db = &expect_context::<crate::state::AppState>().db;
    let o = |s: Option<String>| s.unwrap_or_default();
    let items = sqlx::query!("SELECT * FROM pricelist_items ORDER BY position, id")
        .fetch_all(db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .into_iter()
        .map(|i| PriceItem {
            id: i.id,
            category: i.category,
            delivery_rate: i.delivery_rate != 0,
            values: [
                ("item_name", o(i.item_name)),
                ("item_name_de", o(i.item_name_de)),
                ("details", o(i.details)),
                ("details_de", o(i.details_de)),
                ("subcategory", o(i.subcategory)),
                ("subcategory_de", o(i.subcategory_de)),
                ("price", i.price),
                ("price_de", o(i.price_de)),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect(),
        })
        .collect();
    Ok((items, v.shop.czk_per_km))
}

#[component]
pub fn PricelistPage() -> impl IntoView {
    view! {
        <AdminLayout>
            <div class="admin-toolbar">
                <h1 class="admin-page-title">"Ceník"</h1>
                <a href="/kontakt#cenik" target="_blank" rel="noopener" class="btn-outline-dark">"Zobrazit ceník na webu ↗"</a>
            </div>
            <p class="admin-page-subtitle">"Ceník sortimentu a služeb na stránce Kontakt. Změny se na webu projeví ihned po uložení."</p>
            {loaded(load_pricelist(), |(items, per_km): (Vec<PriceItem>, i64)| {
                PRICELIST.iter().map(|(category, fields)| {
                    let items: Vec<&PriceItem> = items.iter().filter(|i| i.category == *category).collect();
                    let single = *category == "kamenivo";
                    let can_add = !single || items.is_empty();
                    let new_form = format!("new-pricelist-item-{category}-form");
                    view! {
                        <div class="admin-box admin-option-group" id=*category>
                            <h2>{section_title(category)}</h2>
                            <p class="admin-form-hint">{hint(category)}</p>
                            {items.iter().map(|i| view! { <form action=format!("/admin/pricelist_items/{}", i.id) method="post" id=format!("pricelist-item-{}-form", i.id)></form> }).collect_view()}
                            {can_add.then(|| view! { <form action="/admin/pricelist_items" method="post" id=new_form.clone()></form> })}
                            <table class="admin-table">
                                <thead>
                                    <tr>
                                        <th>"Pořadí"</th>
                                        {fields.iter().map(|f| view! { <th>{field_label(category, f)}" (CZ)"</th><th>{field_label(category, f)}" (DE)"</th> }).collect_view()}
                                        <th></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {items.iter().map(|i| {
                                        let form_id = format!("pricelist-item-{}-form", i.id);
                                        view! {
                                            <tr>
                                                <td class="admin-option-move">
                                                    {(!single).then(|| view! {
                                                        <form class="button_to" method="post" action=format!("/admin/pricelist_items/{}/move_up", i.id)><button class="admin-move-btn" aria-label="Posunout nahoru" type="submit">"↑"</button></form>
                                                        <form class="button_to" method="post" action=format!("/admin/pricelist_items/{}/move_down", i.id)><button class="admin-move-btn" aria-label="Posunout dolů" type="submit">"↓"</button></form>
                                                    })}
                                                </td>
                                                {fields.iter().map(|f| {
                                                    let label = field_label(category, f);
                                                    if *f == "price" && i.delivery_rate {
                                                        view! {
                                                            <td colspan="2">
                                                                <strong>{format!("{per_km} Kč / km")}</strong>
                                                                <p class="admin-form-hint">"Cena dopravy z e-shopu, mění se v "<a href="/admin/doprava/edit">"Doprava"</a>"."</p>
                                                            </td>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <td><input type="text" name=format!("pricelist_item[{f}]") value=i.values.get(*f).cloned().unwrap_or_default() form=form_id.clone() aria-label=format!("{label} (CZ)")/></td>
                                                            <td><input type="text" name=format!("pricelist_item[{f}_de]") value=i.values.get(&format!("{f}_de")).cloned().unwrap_or_default() form=form_id.clone() aria-label=format!("{label} (DE)")/></td>
                                                        }.into_any()
                                                    }
                                                }).collect_view()}
                                                <td class="admin-option-actions">
                                                    <input type="submit" value="Uložit" form=form_id.clone() class="btn-outline-dark"/>
                                                    <ConfirmButton action=format!("/admin/pricelist_items/{}/smazat", i.id) text="Smazat" confirm="Opravdu smazat tuto položku ceníku?"/>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                    {can_add.then(|| view! {
                                        <tr>
                                            <td><input type="hidden" name="pricelist_item[category]" value=*category form=new_form.clone()/></td>
                                            {fields.iter().map(|f| {
                                                let label = field_label(category, f);
                                                view! {
                                                    <td><input type="text" name=format!("pricelist_item[{f}]") placeholder=format!("{label} (CZ)") form=new_form.clone() aria-label=format!("Nová položka: {label} (CZ)")/></td>
                                                    <td><input type="text" name=format!("pricelist_item[{f}_de]") placeholder=format!("{label} (DE)") form=new_form.clone() aria-label=format!("Nová položka: {label} (DE)")/></td>
                                                }
                                            }).collect_view()}
                                            <td><input type="submit" value="+ Přidat" form=new_form.clone() class="btn-primary"/></td>
                                        </tr>
                                    })}
                                </tbody>
                            </table>
                        </div>
                    }
                }).collect_view()
            })}
        </AdminLayout>
    }
}

/// The inquiry form's dropdowns, per category.
pub const INQUIRY_FIELDS: [(&str, &[&str]); 2] = [("palivove", &["varianta", "druh", "delka", "mnozstvi"]), ("stavebni", &["polozka", "delka", "vyska", "sirka"])];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptionRow {
    id: i64,
    category: String,
    field: String,
    value: String,
    value_de: String,
}

#[server(endpoint = "admin/inquiry-options")]
pub async fn load_options() -> Result<Vec<OptionRow>, ServerFnError> {
    super::require_admin()?;
    let db = &expect_context::<crate::state::AppState>().db;
    Ok(sqlx::query!("SELECT id, category, field, value, value_de FROM inquiry_form_options ORDER BY position, id")
        .fetch_all(db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .into_iter()
        .map(|o| OptionRow { id: o.id, category: o.category, field: o.field, value: o.value, value_de: o.value_de.unwrap_or_default() })
        .collect())
}

#[component]
pub fn OptionsPage() -> impl IntoView {
    view! {
        <AdminLayout>
            <div class="admin-toolbar"><h1 class="admin-page-title">"Formulář poptávky"</h1></div>
            <p class="admin-page-subtitle">"Možnosti v rozbalovacích seznamech na kalkulačce v kontaktním formuláři. První možnost v seznamu je přednastavená (preset)."</p>
            {loaded(load_options(), |options: Vec<OptionRow>| {
                INQUIRY_FIELDS.iter().map(|(category, fields)| view! {
                    <h2 class="admin-section-title">{if *category == "palivove" { "Palivové dřevo" } else { "Stavební řezivo" }}</h2>
                    {fields.iter().map(|field| {
                        let rows: Vec<&OptionRow> = options.iter().filter(|o| o.category == *category && o.field == *field).collect();
                        let new_form = format!("new-option-{category}-{field}-form");
                        view! {
                            <div class="admin-box admin-option-group" id=format!("{category}-{field}")>
                                <h2>{super::orders::field_label(field)}</h2>
                                {rows.iter().map(|o| view! { <form action=format!("/admin/inquiry_form_options/{}", o.id) method="post" id=format!("option-{}-form", o.id)></form> }).collect_view()}
                                <form action="/admin/inquiry_form_options" method="post" id=new_form.clone()></form>
                                <table class="admin-table">
                                    <thead><tr><th>"Pořadí"</th><th>"Hodnota (CZ)"</th><th>"Hodnota (DE)"</th><th></th></tr></thead>
                                    <tbody>
                                        {rows.iter().map(|o| {
                                            let form_id = format!("option-{}-form", o.id);
                                            view! {
                                                <tr>
                                                    <td class="admin-option-move">
                                                        <form class="button_to" method="post" action=format!("/admin/inquiry_form_options/{}/move_up", o.id)><button class="admin-move-btn" type="submit">"↑"</button></form>
                                                        <form class="button_to" method="post" action=format!("/admin/inquiry_form_options/{}/move_down", o.id)><button class="admin-move-btn" type="submit">"↓"</button></form>
                                                    </td>
                                                    <td><input type="text" name="inquiry_form_option[value]" value=o.value.clone() form=form_id.clone()/></td>
                                                    <td><input type="text" name="inquiry_form_option[value_de]" value=o.value_de.clone() form=form_id.clone()/></td>
                                                    <td class="admin-option-actions">
                                                        <input type="submit" value="Uložit" form=form_id class="btn-outline-dark"/>
                                                        <ConfirmButton action=format!("/admin/inquiry_form_options/{}/smazat", o.id) text="Smazat" confirm="Opravdu smazat tuto možnost?"/>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                        <tr>
                                            <td></td>
                                            <td>
                                                <input type="hidden" name="inquiry_form_option[category]" value=*category form=new_form.clone()/>
                                                <input type="hidden" name="inquiry_form_option[field]" value=*field form=new_form.clone()/>
                                                <input type="text" name="inquiry_form_option[value]" placeholder="Nová možnost (CZ)" form=new_form.clone()/>
                                            </td>
                                            <td><input type="text" name="inquiry_form_option[value_de]" placeholder="Nová možnost (DE)" form=new_form.clone()/></td>
                                            <td><input type="submit" value="+ Přidat možnost" form=new_form class="btn-primary"/></td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        }
                    }).collect_view()}
                }).collect_view()
            })}
        </AdminLayout>
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromoRow {
    id: i64,
    title: String,
    price: String,
    link: String,
    this_week: bool,
}

#[server(endpoint = "admin/promos")]
pub async fn load_promos() -> Result<Vec<PromoRow>, ServerFnError> {
    super::require_admin()?;
    let db = &expect_context::<crate::state::AppState>().db;
    let rows = sqlx::query!("SELECT id, title, price, link FROM promos ORDER BY id").fetch_all(db).await.map_err(|e| ServerFnError::new(e.to_string()))?;
    // The home page shows promo number (ISO week % count), as before.
    let week = time::OffsetDateTime::now_utc().date().iso_week() as usize;
    let pick = (!rows.is_empty()).then(|| week % rows.len());
    Ok(rows
        .into_iter()
        .enumerate()
        .map(|(i, p)| PromoRow {
            id: p.id,
            title: p.title.unwrap_or_default(),
            price: p.price.unwrap_or_default(),
            link: p.link.unwrap_or_default(),
            this_week: pick == Some(i),
        })
        .collect())
}

#[component]
pub fn PromosPage() -> impl IntoView {
    view! {
        <AdminLayout>
            <div class="admin-toolbar">
                <h1 class="admin-page-title">"Doporučujeme (hero)"</h1>
                <a href="/admin/promos/new" class="btn-primary">"+ Nové doporučení"</a>
            </div>
            <p class="admin-form-hint">"Nabídka na hlavní stránce se automaticky střídá každý týden mezi všemi níže uvedenými záznamy (podle pořadí vytvoření). Pro rotaci stačí mít alespoň 2 záznamy."</p>
            {loaded(load_promos(), |rows: Vec<PromoRow>| if rows.is_empty() {
                view! { <div class="admin-empty-state">"Zatím žádná doporučení. Bez záznamu se na hlavní stránce nezobrazí žádné doporučení."</div> }.into_any()
            } else {
                view! {
                    <table class="admin-table">
                        <thead><tr><th>"Název"</th><th>"Cena"</th><th>"Odkaz"</th><th>"Tento týden"</th><th></th></tr></thead>
                        <tbody>
                            {rows.into_iter().map(|p| view! {
                                <tr>
                                    <td><strong>{p.title}</strong></td>
                                    <td>{p.price}</td>
                                    <td>{p.link}</td>
                                    <td>{if p.this_week { "✓ Aktivní" } else { "" }}</td>
                                    <td>
                                        <div class="admin-row-actions">
                                            <a href=format!("/admin/promos/{}/edit", p.id) class="admin-edit-btn">"Upravit"</a>
                                            <ConfirmButton action=format!("/admin/promos/{}/smazat", p.id) text="Smazat" confirm="Opravdu smazat tuto akční nabídku?"/>
                                        </div>
                                    </td>
                                </tr>
                            }).collect_view()}
                        </tbody>
                    </table>
                }.into_any()
            })}
        </AdminLayout>
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PromoForm {
    id: Option<i64>,
    values: Values,
    image: Option<String>,
}

#[cfg(feature = "ssr")]
const PROMO_FIELDS: [&str; 11] = ["title", "title_de", "feature_1", "feature_1_de", "feature_2", "feature_2_de", "feature_3", "feature_3_de", "price", "price_de", "link"];

#[server(endpoint = "admin/promo")]
pub async fn load_promo(id: Option<i64>) -> Result<Option<PromoForm>, ServerFnError> {
    super::require_admin()?;
    let Some(id) = id else { return Ok(Some(PromoForm::default())) };
    let db = &expect_context::<crate::state::AppState>().db;
    let Some(p) = sqlx::query!("SELECT * FROM promos WHERE id = ?", id).fetch_optional(db).await.map_err(|e| ServerFnError::new(e.to_string()))? else {
        return Ok(None);
    };
    let o = |s: Option<String>| s.unwrap_or_default();
    let values = PROMO_FIELDS
        .iter()
        .zip([o(p.title), o(p.title_de), o(p.feature_1), o(p.feature_1_de), o(p.feature_2), o(p.feature_2_de), o(p.feature_3), o(p.feature_3_de), o(p.price), o(p.price_de), o(p.link)])
        .map(|(k, v)| ((*k).to_owned(), v))
        .collect();
    Ok(Some(PromoForm { id: Some(id), values, image: p.image }))
}

#[component]
pub fn PromoPage() -> impl IntoView {
    let id: Option<i64> = use_params_map().with_untracked(|p| p.get("id").and_then(|id| id.parse().ok()));
    view! {
        <AdminLayout>
            <a href="/admin/promos" class="admin-back-link">"← Zpět na přehled"</a>
            {loaded(load_promo(id), |f: Option<PromoForm>| {
                let Some(f) = f else { return view! { <p>"Doporučení nenalezeno."</p> }.into_any() };
                let v = visitor();
                let (values, errors) = values_for(&v, f.values.clone());
                let form = Form { model: "promo", values: &values };
                let action = f.id.map_or_else(|| "/admin/promos".to_owned(), |id| format!("/admin/promos/{id}"));
                view! {
                    <h1 class="admin-page-title">{match f.id {
                        Some(_) => format!("Upravit doporučení: {}", f.values.get("title").cloned().unwrap_or_default()),
                        None => "Nové doporučení".into(),
                    }}</h1>
                    <form action=action method="post" enctype="multipart/form-data" class="admin-form">
                        <Errors errors=errors/>
                        <div class="admin-form-grid">
                            {form.image("image", "Obrázek", f.image.clone(), "Nahrajte novou fotku pro nahrazení aktuální. Necháte-li prázdné, zůstane aktuální obrázek.")}
                            {form.input("text", "link", "Odkaz tlačítka", None, false, Some("/eshop/akat"))}
                        </div>
                        <h2 class="admin-section-title">"Český obsah"</h2>
                        <div class="admin-form-grid">
                            {form.text("title", "Název", None, true)}
                            {form.text("feature_1", "Vlastnost 1", None, false)}
                            {form.text("feature_2", "Vlastnost 2", None, false)}
                            {form.text("feature_3", "Vlastnost 3", None, false)}
                            {form.input("text", "price", "Cena", None, false, Some("od 1500 / 1 PRMS"))}
                        </div>
                        <h2 class="admin-section-title">"Německý obsah"</h2>
                        <p class="admin-form-hint" style="margin-bottom: 14px;">"Pokud necháte prázdné, zobrazí se automaticky český text."</p>
                        <div class="admin-form-grid">
                            {form.text("title_de", "Název (DE)", None, true)}
                            {form.text("feature_1_de", "Vlastnost 1 (DE)", None, false)}
                            {form.text("feature_2_de", "Vlastnost 2 (DE)", None, false)}
                            {form.text("feature_3_de", "Vlastnost 3 (DE)", None, false)}
                            {form.input("text", "price_de", "Cena (DE)", None, false, Some("ab 1500 / 1 Rm"))}
                        </div>
                        <div class="admin-form-actions">
                            <input type="submit" value="Uložit akční nabídku" class="btn-primary"/>
                            <a href="/admin/promos" class="btn-outline-dark">"Zpět na přehled"</a>
                        </div>
                    </form>
                }.into_any()
            })}
        </AdminLayout>
    }
}

#[cfg(feature = "ssr")]
pub mod handlers {
    use super::super::form::parse::{fields, multipart, text};
    use super::super::handlers::{done, failed, guard, refused, save_upload};
    use super::{INQUIRY_FIELDS, PRICELIST};
    use crate::state::AppState;
    use crate::visitor::Visitor;
    use axum::extract::{Multipart, Path, State};
    use axum::response::Response;
    use axum::{Extension, Form};
    use tower_sessions::Session;

    type Pairs = Vec<(String, String)>;

    macro_rules! admin {
        ($session:expr, $v:expr) => {
            if let Err(r) = guard(&$session, &$v).await {
                return r;
            }
        };
    }

    /// Why a price list row can't be saved (as the old validations said it).
    fn pricelist_problems(category: &str, values: &super::Values) -> Vec<String> {
        let mut problems = Vec::new();
        let fields = PRICELIST.iter().find(|(c, _)| *c == category).map_or(&[][..], |(_, f)| *f);
        if fields.contains(&"item_name") && text(values, "item_name").is_none() {
            problems.push("Název je povinné pole".to_owned());
        }
        if category.starts_with("palivove") && text(values, "subcategory").is_none() {
            problems.push("Dřevina je povinné pole".to_owned());
        }
        problems
    }

    pub async fn create_price(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Form(pairs): Form<Pairs>) -> Response {
        admin!(session, v);
        let values = fields(&pairs, "pricelist_item");
        let category = text(&values, "category").unwrap_or_default();
        let back = format!("/admin/pricelist_items#{category}");
        if !PRICELIST.iter().any(|(c, _)| *c == category) {
            return failed(&session, "/admin/pricelist_items", "Kategorie má neplatnou hodnotu").await;
        }
        let mut problems = pricelist_problems(&category, &values);
        let price = text(&values, "price");
        if price.is_none() {
            problems.push("Cena je povinné pole".to_owned());
        }
        if !problems.is_empty() {
            return failed(&session, &back, problems.join(", ")).await;
        }
        let t = |f: &str| text(&values, f);
        let (name, name_de, details, details_de, sub, sub_de, price_de) = (t("item_name"), t("item_name_de"), t("details"), t("details_de"), t("subcategory"), t("subcategory_de"), t("price_de"));
        let _ = sqlx::query!(
            "INSERT INTO pricelist_items (category, position, item_name, item_name_de, details, details_de, subcategory, subcategory_de, price, price_de)
             VALUES (?, (SELECT COALESCE(MAX(position), 0) + 1 FROM pricelist_items WHERE category = ?), ?, ?, ?, ?, ?, ?, ?, ?)",
            category, category, name, name_de, details, details_de, sub, sub_de, price, price_de
        )
        .execute(&state.db)
        .await;
        done(&session, &back, "Položka byla přidána.").await
    }

    pub async fn update_price(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Path(id): Path<i64>, Form(pairs): Form<Pairs>) -> Response {
        admin!(session, v);
        let Some(row) = sqlx::query!("SELECT category, delivery_rate, price FROM pricelist_items WHERE id = ?", id).fetch_optional(&state.db).await.ok().flatten() else {
            return failed(&session, "/admin/pricelist_items", "Položka nenalezena.").await;
        };
        let values = fields(&pairs, "pricelist_item");
        let back = format!("/admin/pricelist_items#{}", row.category);
        let mut problems = pricelist_problems(&row.category, &values);
        // The delivery row's price isn't in the form (it's the e-shop's).
        let price = if row.delivery_rate != 0 { Some(row.price) } else { text(&values, "price") };
        if price.is_none() {
            problems.push("Cena je povinné pole".to_owned());
        }
        if !problems.is_empty() {
            return failed(&session, &back, problems.join(", ")).await;
        }
        let t = |f: &str| text(&values, f);
        let (name, name_de, details, details_de, sub, sub_de, price_de) = (t("item_name"), t("item_name_de"), t("details"), t("details_de"), t("subcategory"), t("subcategory_de"), t("price_de"));
        let _ = sqlx::query!(
            "UPDATE pricelist_items SET item_name = ?, item_name_de = ?, details = ?, details_de = ?, subcategory = ?, subcategory_de = ?, price = ?, price_de = ?,
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?",
            name, name_de, details, details_de, sub, sub_de, price, price_de, id
        )
        .execute(&state.db)
        .await;
        done(&session, &back, "Položka byla uložena.").await
    }

    pub async fn delete_price(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Path(id): Path<i64>) -> Response {
        admin!(session, v);
        let category = sqlx::query_scalar!("DELETE FROM pricelist_items WHERE id = ? RETURNING category", id).fetch_optional(&state.db).await.ok().flatten().unwrap_or_default();
        done(&session, &format!("/admin/pricelist_items#{category}"), "Položka byla smazána.").await
    }

    /// The ids of a group in order, with `id` swapped with its neighbour
    /// (`step` -1 up, +1 down); `None` when there's nowhere to move.
    fn swapped(mut ids: Vec<i64>, id: i64, step: i64) -> Option<Vec<i64>> {
        let index = ids.iter().position(|x| *x == id)?;
        let target = usize::try_from(i64::try_from(index).ok()? + step).ok().filter(|t| *t < ids.len())?;
        ids.swap(index, target);
        Some(ids)
    }

    /// Moves a price list row up or down within its category (renumbered 0, 1, 2...).
    async fn move_price_row(state: &AppState, id: i64, step: i64) -> Result<(), sqlx::Error> {
        let ids = sqlx::query_scalar!(
            r#"SELECT id AS "id!" FROM pricelist_items WHERE category = (SELECT category FROM pricelist_items WHERE id = ?) ORDER BY position, id"#,
            id
        )
        .fetch_all(&state.db)
        .await?;
        let Some(ids) = swapped(ids, id, step) else { return Ok(()) };
        let mut tx = state.db.begin().await?;
        for (position, row) in (0_i64..).zip(ids) {
            sqlx::query!("UPDATE pricelist_items SET position = ? WHERE id = ?", position, row).execute(&mut *tx).await?;
        }
        tx.commit().await
    }

    /// Moves an inquiry form option up or down within its dropdown.
    async fn move_option_row(state: &AppState, id: i64, step: i64) -> Result<(), sqlx::Error> {
        let ids = sqlx::query_scalar!(
            r#"SELECT o.id AS "id!" FROM inquiry_form_options o JOIN inquiry_form_options me ON me.id = ?
               WHERE o.category = me.category AND o.field = me.field ORDER BY o.position, o.id"#,
            id
        )
        .fetch_all(&state.db)
        .await?;
        let Some(ids) = swapped(ids, id, step) else { return Ok(()) };
        let mut tx = state.db.begin().await?;
        for (position, row) in (0_i64..).zip(ids) {
            sqlx::query!("UPDATE inquiry_form_options SET position = ? WHERE id = ?", position, row).execute(&mut *tx).await?;
        }
        tx.commit().await
    }

    pub async fn move_price(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Path((id, direction)): Path<(i64, String)>) -> Response {
        admin!(session, v);
        let step = if direction == "move_up" { -1 } else { 1 };
        let category = sqlx::query_scalar!("SELECT category FROM pricelist_items WHERE id = ?", id).fetch_optional(&state.db).await.ok().flatten().unwrap_or_default();
        if let Err(e) = move_price_row(&state, id, step).await {
            tracing::warn!("not moved: {e}");
        }
        axum::response::IntoResponse::into_response(axum::response::Redirect::to(&format!("/admin/pricelist_items#{category}")))
    }

    pub async fn create_option(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Form(pairs): Form<Pairs>) -> Response {
        admin!(session, v);
        let values = fields(&pairs, "inquiry_form_option");
        let (category, field) = (text(&values, "category").unwrap_or_default(), text(&values, "field").unwrap_or_default());
        let valid = INQUIRY_FIELDS.iter().any(|(c, fs)| *c == category && fs.contains(&field.as_str()));
        let Some(value) = text(&values, "value").filter(|_| valid) else {
            let problem = if valid { "Hodnota je povinné pole" } else { "Pole nepatří k této kategorii" };
            return failed(&session, "/admin/inquiry_form_options", problem).await;
        };
        let value_de = text(&values, "value_de");
        let _ = sqlx::query!(
            "INSERT INTO inquiry_form_options (category, field, value, value_de, position)
             VALUES (?, ?, ?, ?, (SELECT COALESCE(MAX(position), 0) + 1 FROM inquiry_form_options WHERE category = ? AND field = ?))",
            category, field, value, value_de, category, field
        )
        .execute(&state.db)
        .await;
        done(&session, &format!("/admin/inquiry_form_options#{category}-{field}"), "Možnost byla přidána.").await
    }

    pub async fn update_option(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Path(id): Path<i64>, Form(pairs): Form<Pairs>) -> Response {
        admin!(session, v);
        let Some(row) = sqlx::query!("SELECT category, field FROM inquiry_form_options WHERE id = ?", id).fetch_optional(&state.db).await.ok().flatten() else {
            return failed(&session, "/admin/inquiry_form_options", "Možnost nenalezena.").await;
        };
        let back = format!("/admin/inquiry_form_options#{}-{}", row.category, row.field);
        let values = fields(&pairs, "inquiry_form_option");
        let Some(value) = text(&values, "value") else { return failed(&session, &back, "Hodnota je povinné pole").await };
        let value_de = text(&values, "value_de");
        let _ = sqlx::query!("UPDATE inquiry_form_options SET value = ?, value_de = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?", value, value_de, id)
            .execute(&state.db)
            .await;
        done(&session, &back, "Možnost byla uložena.").await
    }

    pub async fn delete_option(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Path(id): Path<i64>) -> Response {
        admin!(session, v);
        let _ = sqlx::query!("DELETE FROM inquiry_form_options WHERE id = ?", id).execute(&state.db).await;
        done(&session, "/admin/inquiry_form_options", "Možnost byla smazána.").await
    }

    pub async fn move_option(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Path((id, direction)): Path<(i64, String)>) -> Response {
        admin!(session, v);
        let step = if direction == "move_up" { -1 } else { 1 };
        if let Err(e) = move_option_row(&state, id, step).await {
            tracing::warn!("not moved: {e}");
        }
        let group = sqlx::query!("SELECT category, field FROM inquiry_form_options WHERE id = ?", id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
            .map(|o| format!("{}-{}", o.category, o.field))
            .unwrap_or_default();
        axum::response::IntoResponse::into_response(axum::response::Redirect::to(&format!("/admin/inquiry_form_options#{group}")))
    }

    async fn save_promo(state: &AppState, session: &Session, id: Option<i64>, form: Multipart) -> Response {
        let (pairs, files) = multipart(form).await;
        let values = fields(&pairs, "promo");
        let form_path = id.map_or_else(|| "/admin/promos/new".to_owned(), |id| format!("/admin/promos/{id}/edit"));
        let mut image = None;
        if let Some((_, bytes)) = files.iter().find(|(name, _)| name == "promo[image]") {
            match save_upload(state, bytes).await {
                Ok(path) => image = Some(path),
                Err(problem) => return refused(session, &form_path, values, vec![problem]).await,
            }
        }
        let t = |f: &str| text(&values, f);
        let (title, title_de, f1, f1_de, f2, f2_de, f3, f3_de, price, price_de, link) =
            (t("title"), t("title_de"), t("feature_1"), t("feature_1_de"), t("feature_2"), t("feature_2_de"), t("feature_3"), t("feature_3_de"), t("price"), t("price_de"), t("link"));
        let saved = match id {
            Some(id) => sqlx::query!(
                "UPDATE promos SET title = ?, title_de = ?, feature_1 = ?, feature_1_de = ?, feature_2 = ?, feature_2_de = ?, feature_3 = ?, feature_3_de = ?,
                        price = ?, price_de = ?, link = ?, image = COALESCE(?, image), updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?",
                title, title_de, f1, f1_de, f2, f2_de, f3, f3_de, price, price_de, link, image, id
            )
            .execute(&state.db)
            .await,
            None => sqlx::query!(
                "INSERT INTO promos (title, title_de, feature_1, feature_1_de, feature_2, feature_2_de, feature_3, feature_3_de, price, price_de, link, image)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                title, title_de, f1, f1_de, f2, f2_de, f3, f3_de, price, price_de, link, image
            )
            .execute(&state.db)
            .await,
        };
        match saved {
            Ok(_) => done(session, "/admin/promos", if id.is_some() { "Doporučení bylo uloženo." } else { "Doporučení bylo vytvořeno." }).await,
            Err(e) => refused(session, &form_path, values, vec![e.to_string()]).await,
        }
    }

    pub async fn create_promo(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, form: Multipart) -> Response {
        admin!(session, v);
        save_promo(&state, &session, None, form).await
    }

    pub async fn update_promo(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Path(id): Path<i64>, form: Multipart) -> Response {
        admin!(session, v);
        save_promo(&state, &session, Some(id), form).await
    }

    pub async fn delete_promo(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Path(id): Path<i64>) -> Response {
        admin!(session, v);
        let _ = sqlx::query!("DELETE FROM promos WHERE id = ?", id).execute(&state.db).await;
        done(&session, "/admin/promos", "Doporučení bylo smazáno.").await
    }
}
