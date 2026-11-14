//! Admin: delivery price, VAT rules and discount codes.

use super::form::{Errors, Form, Values, values_for};
use super::{AdminLayout, ConfirmButton, czk};
use crate::pages::loaded;
use crate::visitor::visitor;
use domain::Category;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

#[component]
pub fn DeliveryPage() -> impl IntoView {
    let v = visitor();
    let per_km = v.shop.czk_per_km;
    let refused = v
        .admin_refused
        .as_ref()
        .filter(|r| r.path == v.path)
        .cloned();
    let value = refused
        .as_ref()
        .and_then(|r| r.values.get("value").cloned())
        .unwrap_or_else(|| per_km.to_string());
    view! {
        <AdminLayout>
            <h1 class="admin-page-title">"Doprava"</h1>
            <form action="/admin/doprava" method="post" class="admin-form">
                {refused.map(|_| view! {
                    <div class="admin-flash admin-flash-alert"><strong>"Cenu nelze uložit:"</strong>" zadejte celé číslo od 1 do 1 000."</div>
                })}
                <div class="form-group">
                    <label for="setting_value">"Cena dopravy na adresu (Kč za km, včetně DPH)"</label>
                    <input type="number" name="setting[value]" id="setting_value" value=value min="1" max="1000" step="1" required/>
                    <p class="admin-form-hint">
                        "Počítá se tam i zpět mezi Derflicemi a adresou zákazníka, jednou jízdou za celou objednávku. "
                        {format!("Např. adresa 50 km daleko: 2 × 50 km × {per_km} Kč = {}.", czk(100 * per_km))}
                    </p>
                    <p class="admin-form-hint">
                        "Nová cena platí hned pro nové objednávky a ukáže se v e-shopu, v pokladně, v ceníku na stránce Kontakt i v obchodních podmínkách. "
                        "Už odeslané objednávky si ponechají cenu, se kterou byly objednány."
                    </p>
                </div>
                <div class="admin-form-actions">
                    <input type="submit" value="Uložit cenu dopravy" class="btn-primary"/>
                </div>
            </form>
        </AdminLayout>
    }
}

#[component]
pub fn VatPage() -> impl IntoView {
    let v = visitor();
    let refused = v
        .admin_refused
        .clone()
        .filter(|r| r.path.starts_with("/admin/dph#"));
    let rows = Category::ALL
        .iter()
        .map(|category| {
            let rule = v.shop.vat.get(*category);
            let form_id = format!("vat-{}", category.as_str());
            let label = category.admin_label();
            let error = refused.as_ref().filter(|r| r.path == format!("/admin/dph#{}", category.as_str())).map(|r| r.errors.join(", "));
            view! {
                <tr>
                    <td>
                        <strong>{label}</strong>
                        {error.map(|e| view! { <br/><small class="admin-error">{e}</small> })}
                    </td>
                    <td>
                        <select name="vat_rule[prices_include_vat]" form=form_id.clone() aria-label=format!("Ceny v katalogu – {label}")>
                            <option value="true" selected=rule.prices_include_vat>"včetně DPH"</option>
                            <option value="false" selected=!rule.prices_include_vat>"bez DPH"</option>
                        </select>
                    </td>
                    <td><input type="number" name="vat_rule[rate_cz]" form=form_id.clone() value=rule.rate_cz min="0" max="99" step="1" required aria-label=format!("DPH Česko – {label}")/>" %"</td>
                    <td><input type="number" name="vat_rule[rate_at]" form=form_id.clone() value=rule.rate_at min="0" max="99" step="1" required aria-label=format!("DPH Rakousko – {label}")/>" %"</td>
                    <td>
                        <form id=form_id action=format!("/admin/dph/{}", category.as_str()) method="post">
                            <input type="submit" value="Uložit" class="btn-primary"/>
                        </form>
                    </td>
                </tr>
            }
        })
        .collect_view();
    view! {
        <AdminLayout>
            <h1 class="admin-page-title">"DPH"</h1>
            <p class="admin-form-hint">
                "Zákazník platí v Česku i v Rakousku stejnou konečnou cenu. Liší se jen to, kolik z ní je DPH: "
                "při dovozu do Rakouska se počítá rakouská sazba, při dovozu po Česku a při osobním odběru v Derflicích česká. "
                "Doprava a sleva se rozpočítají podle podílu zboží (u palivového dřeva tedy doprava do Rakouska také 13 %)."
            </p>
            <p class="admin-form-hint">
                "„Ceny v katalogu bez DPH“ znamená, že k ceně zadané v Katalogu se pro zákazníka přičte česká DPH "
                {format!("(8 500 Kč → {} při 21 %).", czk(domain::Czk(8500).plus_vat(21).get()))}
                " Změna platí hned pro nové objednávky; odeslané objednávky si ponechají DPH, se kterou byly objednány."
            </p>
            <table class="admin-table">
                <thead><tr><th>"Kategorie"</th><th>"Ceny v katalogu"</th><th>"DPH Česko"</th><th>"DPH Rakousko"</th><th></th></tr></thead>
                <tbody>{rows}</tbody>
            </table>
        </AdminLayout>
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeRow {
    id: i64,
    code: String,
    note: Option<String>,
    label: String,
    conditions: Vec<String>,
    used: String,
    state: String,
}

#[server(endpoint = "admin/discount-codes")]
pub async fn load_codes() -> Result<Vec<CodeRow>, ServerFnError> {
    super::require_admin()?;
    let db = &expect_context::<crate::state::AppState>().db;
    let today = time::OffsetDateTime::now_utc().date();
    let rows =
        sqlx::query!(r#"SELECT id AS "id!", code FROM discount_codes ORDER BY created_at DESC"#)
            .fetch_all(db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    let mut out = Vec::new();
    for row in rows {
        let Some(code) = crate::session::discount_code(db, &row.code)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
        else {
            continue;
        };
        let note = sqlx::query_scalar!("SELECT note FROM discount_codes WHERE id = ?", row.id)
            .fetch_one(db)
            .await
            .ok()
            .flatten();
        use domain::discount::Unusable;
        let mut conditions: Vec<String> = [
            code.min_order.map(|m| format!("od {m}")),
            code.valid_until
                .map(|d| format!("do {}. {}. {}", d.day(), d.month() as u8, d.year())),
        ]
        .into_iter()
        .flatten()
        .collect();
        if conditions.is_empty() {
            conditions.push("—".into());
        }
        out.push(CodeRow {
            id: row.id,
            note: note.filter(|n| !n.is_empty()),
            label: code.label(),
            used: format!(
                "{}{}×",
                code.times_used,
                code.usage_limit
                    .map(|l| format!(" / {l}"))
                    .unwrap_or_default()
            ),
            state: match code.unusable_reason(domain::Czk(i64::MAX), today) {
                None => "✓ Platí",
                Some(Unusable::Inactive) => "Vypnutý",
                Some(Unusable::Expired) => "Vypršel",
                Some(_) => "Vyčerpaný",
            }
            .to_owned(),
            conditions,
            code: code.code,
        });
    }
    Ok(out)
}

#[component]
pub fn CodesPage() -> impl IntoView {
    view! {
        <AdminLayout>
            <div class="admin-toolbar">
                <h1 class="admin-page-title">"Slevové kódy"</h1>
                <a href="/admin/slevove-kody/new" class="btn-primary">"+ Nový slevový kód"</a>
            </div>
            <p class="admin-form-hint">"Zákazník kód zadá v košíku. Sleva se počítá jen ze zboží, ne z dopravy. Na jednu objednávku jde použít jen jeden kód."</p>
            {loaded(load_codes(), |rows: Vec<CodeRow>| if rows.is_empty() {
                view! { <div class="admin-empty-state">"Zatím žádné slevové kódy."</div> }.into_any()
            } else {
                view! {
                    <table class="admin-table">
                        <thead><tr><th>"Kód"</th><th>"Sleva"</th><th>"Podmínky"</th><th>"Použito"</th><th>"Stav"</th><th></th></tr></thead>
                        <tbody>
                            {rows.into_iter().map(|c| {
                                let confirm = format!("Opravdu smazat slevový kód {}? Odeslané objednávky si slevu ponechají.", c.code);
                                let last = c.conditions.len() - 1;
                                view! {
                                    <tr>
                                        <td><strong>{c.code.clone()}</strong>{c.note.map(|n| view! { <br/><small>{n}</small> })}</td>
                                        <td>{c.label}</td>
                                        <td>{c.conditions.into_iter().enumerate().map(|(i, x)| view! { {x}{(i < last).then(|| view! { <br/> })} }).collect_view()}</td>
                                        <td>{c.used}</td>
                                        <td>{c.state}</td>
                                        <td>
                                            <div class="admin-row-actions">
                                                <a href=format!("/admin/slevove-kody/{}/edit", c.id) class="admin-edit-btn">"Upravit"</a>
                                                <ConfirmButton action=format!("/admin/slevove-kody/{}/smazat", c.id) text="Smazat" confirm=confirm/>
                                            </div>
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                }.into_any()
            })}
        </AdminLayout>
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CodeForm {
    id: Option<i64>,
    code: String,
    times_used: i64,
    values: Values,
}

#[server(endpoint = "admin/discount-code")]
pub async fn load_code(id: Option<i64>) -> Result<Option<CodeForm>, ServerFnError> {
    super::require_admin()?;
    let Some(id) = id else {
        let values = [("kind", "percent"), ("active", "1")]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        return Ok(Some(CodeForm {
            values,
            ..CodeForm::default()
        }));
    };
    let db = &expect_context::<crate::state::AppState>().db;
    let Some(c) = sqlx::query!("SELECT * FROM discount_codes WHERE id = ?", id)
        .fetch_optional(db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    else {
        return Ok(None);
    };
    let n = |x: Option<i64>| x.map(|n| n.to_string()).unwrap_or_default();
    let values = [
        ("code", c.code.clone()),
        ("note", c.note.unwrap_or_default()),
        ("kind", c.kind),
        ("value", c.value.to_string()),
        ("min_order_czk", n(c.min_order_czk)),
        ("valid_until", c.valid_until.unwrap_or_default()),
        ("usage_limit", n(c.usage_limit)),
        ("active", c.active.to_string()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v))
    .collect();
    Ok(Some(CodeForm {
        id: Some(id),
        code: c.code,
        times_used: c.times_used,
        values,
    }))
}

#[component]
pub fn CodePage() -> impl IntoView {
    let id: Option<i64> =
        use_params_map().with_untracked(|p| p.get("id").and_then(|id| id.parse().ok()));
    view! {
        <AdminLayout>
            <a href="/admin/slevove-kody" class="admin-back-link">"← Zpět na přehled"</a>
            {loaded(load_code(id), |f: Option<CodeForm>| {
                let Some(f) = f else { return view! { <p>"Slevový kód nenalezen."</p> }.into_any() };
                let v = visitor();
                let (values, errors) = values_for(&v, f.values.clone());
                let form = Form { model: "discount_code", values: &values };
                let action = f.id.map_or_else(|| "/admin/slevove-kody".to_owned(), |id| format!("/admin/slevove-kody/{id}"));
                view! {
                    <h1 class="admin-page-title">{match f.id { Some(_) => format!("Slevový kód {}", f.code), None => "Nový slevový kód".into() }}</h1>
                    <form action=action method="post" class="admin-form">
                        <Errors errors=errors/>
                        <div class="admin-form-grid">
                            <div class="form-group">
                                <label for="discount_code_code">"Kód"</label>
                                <input type="text" name="discount_code[code]" id="discount_code_code" value=form.value("code") placeholder="JARO10" maxlength="40" required style="text-transform: uppercase;"/>
                                <p class="admin-form-hint">"Písmena bez diakritiky, číslice, - a _. Velikost písmen nehraje roli."</p>
                            </div>
                            {form.input("text", "note", "Poznámka (jen pro vás)", None, false, Some("Jarní akce pro stálé zákazníky"))}
                            {form.select("kind", "Typ slevy", &[("Procenta ze zboží (%)", "percent"), ("Pevná částka (Kč)", "fixed")], None)}
                            <div class="form-group">
                                <label for="discount_code_value">"Výše slevy"</label>
                                <input type="number" name="discount_code[value]" id="discount_code_value" value=form.value("value") min="1" step="1" required/>
                                <p class="admin-form-hint">"U procent 1–100, u pevné částky v Kč včetně DPH (nikdy víc, než stojí zboží)."</p>
                            </div>
                            <div class="form-group">
                                <label for="discount_code_min_order_czk">"Minimální hodnota zboží (Kč, nepovinné)"</label>
                                <input type="number" name="discount_code[min_order_czk]" id="discount_code_min_order_czk" value=form.value("min_order_czk") min="1" step="1"/>
                            </div>
                            {form.input("date", "valid_until", "Platí do (včetně, nepovinné)", None, false, None)}
                            <div class="form-group">
                                <label for="discount_code_usage_limit">"Kolikrát lze použít (nepovinné)"</label>
                                <input type="number" name="discount_code[usage_limit]" id="discount_code_usage_limit" value=form.value("usage_limit") min="1" step="1"/>
                                <p class="admin-form-hint">{format!("Prázdné = bez omezení. Zatím použito {}×.", f.times_used)}</p>
                            </div>
                        </div>
                        {form.checkbox("active", "Aktivní (zákazníci ho mohou použít)")}
                        <div class="admin-form-actions">
                            <input type="submit" value="Uložit slevový kód" class="btn-primary"/>
                            <a href="/admin/slevove-kody" class="btn-outline-dark">"Zpět na přehled"</a>
                        </div>
                    </form>
                }.into_any()
            })}
        </AdminLayout>
    }
}

#[cfg(feature = "ssr")]
pub mod handlers {
    use super::super::form::parse::{checked, fields, int, text};
    use super::super::handlers::{done, guard, refused};
    use crate::state::AppState;
    use crate::visitor::Visitor;
    use axum::extract::{Path, State};
    use axum::response::Response;
    use axum::{Extension, Form};
    use tower_sessions::Session;

    type Pairs = Vec<(String, String)>;

    pub async fn update_delivery(
        State(state): State<AppState>,
        session: Session,
        Extension(v): Extension<Visitor>,
        Form(pairs): Form<Pairs>,
    ) -> Response {
        if let Err(r) = guard(&session, &v).await {
            return r;
        }
        let values = fields(&pairs, "setting");
        match int(&values, "value") {
            Ok(Some(per_km)) if (1..=domain::delivery::MAX_CZK_PER_KM).contains(&per_km) => {
                let value = per_km.to_string();
                let _ = sqlx::query!(
                    "INSERT INTO settings (key, value) VALUES ('delivery_czk_per_km', ?)
                     ON CONFLICT (key) DO UPDATE SET value = excluded.value, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
                    value
                )
                .execute(&state.db)
                .await;
                done(
                    &session,
                    "/admin/doprava/edit",
                    "Cena dopravy byla uložena.",
                )
                .await
            }
            _ => {
                refused(
                    &session,
                    "/admin/doprava/edit",
                    values,
                    vec!["value".into()],
                )
                .await
            }
        }
    }

    pub async fn update_vat(
        State(state): State<AppState>,
        session: Session,
        Extension(v): Extension<Visitor>,
        Path(category): Path<String>,
        Form(pairs): Form<Pairs>,
    ) -> Response {
        if let Err(r) = guard(&session, &v).await {
            return r;
        }
        let Ok(category) = category.parse::<domain::Category>() else {
            return done(&session, "/admin/dph", "").await;
        };
        let values = fields(&pairs, "vat_rule");
        let rate = |field: &str, label: &str, errors: &mut Vec<String>| match int(&values, field) {
            Ok(Some(r)) if (0..100).contains(&r) => Some(r),
            Ok(Some(_)) => {
                errors.push(format!("{label} musí být mezi 0 a 99"));
                None
            }
            _ => {
                errors.push(format!("{label} musí být číslo"));
                None
            }
        };
        let mut errors = Vec::new();
        let rate_cz = rate("rate_cz", "DPH Česko", &mut errors);
        let rate_at = rate("rate_at", "DPH Rakousko", &mut errors);
        let path = format!("/admin/dph#{}", category.as_str());
        let (Some(rate_cz), Some(rate_at), true) = (rate_cz, rate_at, errors.is_empty()) else {
            return refused(&session, &path, values, errors).await;
        };
        let includes =
            i64::from(values.get("prices_include_vat").map(String::as_str) != Some("false"));
        let category_text = category.as_str();
        let _ = sqlx::query!(
            "INSERT INTO vat_rules (category, rate_cz, rate_at, prices_include_vat) VALUES (?, ?, ?, ?)
             ON CONFLICT (category) DO UPDATE SET rate_cz = excluded.rate_cz, rate_at = excluded.rate_at,
                 prices_include_vat = excluded.prices_include_vat, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
            category_text,
            rate_cz,
            rate_at,
            includes
        )
        .execute(&state.db)
        .await;
        done(
            &session,
            "/admin/dph",
            format!("DPH pro „{}“ bylo uloženo.", category.admin_label()),
        )
        .await
    }

    async fn save_code(
        state: &AppState,
        session: &Session,
        id: Option<i64>,
        pairs: Pairs,
    ) -> Response {
        let values = fields(&pairs, "discount_code");
        let form_path = id.map_or_else(
            || "/admin/slevove-kody/new".to_owned(),
            |id| format!("/admin/slevove-kody/{id}/edit"),
        );
        let mut errors = Vec::new();
        let raw_code = text(&values, "code").unwrap_or_default();
        let code = domain::discount::normalize_code(&raw_code);
        match &code {
            None if raw_code.is_empty() => errors.push("Kód je povinné pole".to_owned()),
            None => errors
                .push("Kód smí obsahovat jen písmena bez diakritiky, číslice, - a _".to_owned()),
            Some(code) => {
                let taken = sqlx::query_scalar!(
                    "SELECT id FROM discount_codes WHERE code = ? COLLATE NOCASE",
                    code
                )
                .fetch_optional(&state.db)
                .await
                .ok()
                .flatten();
                if taken.is_some_and(|other| Some(other) != id) {
                    errors.push("Kód už je použitý".to_owned());
                }
            }
        }
        let kind = text(&values, "kind").filter(|k| k == "percent" || k == "fixed");
        if kind.is_none() {
            errors.push("Typ slevy má neplatnou hodnotu".to_owned());
        }
        let value = match int(&values, "value") {
            Ok(Some(v)) if v > 0 && (kind.as_deref() != Some("percent") || v <= 100) => Some(v),
            Ok(Some(v)) if v > 0 => {
                errors.push("Výše slevy musí být menší nebo rovna 100".to_owned());
                None
            }
            Ok(Some(_)) => {
                errors.push("Výše slevy musí být větší než 0".to_owned());
                None
            }
            _ => {
                errors.push("Výše slevy musí být číslo".to_owned());
                None
            }
        };
        let positive =
            |field: &str, label: &str, errors: &mut Vec<String>| match int(&values, field) {
                Ok(Some(v)) if v > 0 => Some(v),
                Ok(None) => None,
                _ => {
                    errors.push(format!("{label} musí být větší než 0"));
                    None
                }
            };
        let min_order = positive("min_order_czk", "Minimální hodnota zboží", &mut errors);
        let usage_limit = positive("usage_limit", "Počet použití", &mut errors);
        let valid_until = text(&values, "valid_until");
        if valid_until.as_deref().is_some_and(|d| {
            time::Date::parse(d, &time::format_description::well_known::Iso8601::DATE).is_err()
        }) {
            errors.push("Platí do má neplatnou hodnotu".to_owned());
        }
        if !errors.is_empty() {
            return refused(session, &form_path, values, errors).await;
        }
        let (code, kind, value) = (
            code.unwrap_or_default(),
            kind.unwrap_or_default(),
            value.unwrap_or(1),
        );
        let active = i64::from(checked(&values, "active"));
        let note = text(&values, "note");
        let saved = match id {
            Some(id) => sqlx::query!(
                "UPDATE discount_codes SET code = ?, kind = ?, value = ?, active = ?, valid_until = ?, usage_limit = ?, min_order_czk = ?, note = ?,
                        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?",
                code, kind, value, active, valid_until, usage_limit, min_order, note, id
            )
            .execute(&state.db)
            .await
            .map(|_| ()),
            None => sqlx::query!(
                "INSERT INTO discount_codes (code, kind, value, active, valid_until, usage_limit, min_order_czk, note) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                code, kind, value, active, valid_until, usage_limit, min_order, note
            )
            .execute(&state.db)
            .await
            .map(|_| ()),
        };
        match saved {
            Ok(()) => {
                done(
                    session,
                    "/admin/slevove-kody",
                    format!(
                        "Slevový kód {code} byl {}.",
                        if id.is_some() { "uložen" } else { "vytvořen" }
                    ),
                )
                .await
            }
            Err(e) => refused(session, &form_path, values, vec![e.to_string()]).await,
        }
    }

    pub async fn create_code(
        State(state): State<AppState>,
        session: Session,
        Extension(v): Extension<Visitor>,
        Form(pairs): Form<Pairs>,
    ) -> Response {
        if let Err(r) = guard(&session, &v).await {
            return r;
        }
        save_code(&state, &session, None, pairs).await
    }

    pub async fn update_code(
        State(state): State<AppState>,
        session: Session,
        Extension(v): Extension<Visitor>,
        Path(id): Path<i64>,
        Form(pairs): Form<Pairs>,
    ) -> Response {
        if let Err(r) = guard(&session, &v).await {
            return r;
        }
        save_code(&state, &session, Some(id), pairs).await
    }

    pub async fn delete_code(
        State(state): State<AppState>,
        session: Session,
        Extension(v): Extension<Visitor>,
        Path(id): Path<i64>,
    ) -> Response {
        if let Err(r) = guard(&session, &v).await {
            return r;
        }
        let code =
            sqlx::query_scalar!("DELETE FROM discount_codes WHERE id = ? RETURNING code", id)
                .fetch_optional(&state.db)
                .await
                .ok()
                .flatten()
                .unwrap_or_default();
        done(
            &session,
            "/admin/slevove-kody",
            format!("Slevový kód {code} byl smazán."),
        )
        .await
    }
}
