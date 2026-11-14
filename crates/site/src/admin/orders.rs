//! Admin: the overview, orders and inquiries.

use super::{czk, when, AdminLayout, ConfirmButton};
use crate::pages::loaded;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderRow {
    id: i64,
    name: String,
    email: String,
    total: i64,
    pickup: bool,
    bank_transfer: bool,
    created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dashboard {
    products: i64,
    orders: i64,
    this_month: i64,
    last_month: i64,
    recent: Vec<OrderRow>,
}

#[cfg(feature = "ssr")]
async fn order_rows(db: &sqlx::SqlitePool, limit: i64) -> Result<Vec<OrderRow>, sqlx::Error> {
    Ok(sqlx::query!(
        "SELECT id, first_name, last_name, email, total_czk, shipping_method, payment_method, created_at FROM orders ORDER BY created_at DESC LIMIT ?",
        limit
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|o| OrderRow {
        id: o.id,
        name: format!("{} {}", o.first_name, o.last_name).trim().to_owned(),
        email: o.email,
        total: o.total_czk,
        pickup: o.shipping_method == "pickup",
        bank_transfer: o.payment_method == "bank_transfer",
        created_at: o.created_at,
    })
    .collect())
}

#[server(endpoint = "admin/dashboard")]
pub async fn load_dashboard() -> Result<Dashboard, ServerFnError> {
    super::require_admin()?;
    let db = &expect_context::<crate::state::AppState>().db;
    let err = |e: sqlx::Error| ServerFnError::new(e.to_string());
    let products = sqlx::query_scalar!("SELECT COUNT(*) FROM catalog_products").fetch_one(db).await.map_err(err)?;
    let orders = sqlx::query_scalar!("SELECT COUNT(*) FROM orders").fetch_one(db).await.map_err(err)?;
    // Months by Prague time.
    let now = time::OffsetDateTime::now_utc();
    let local = now.to_offset(crate::time_fmt::prague_offset(now));
    let month_start = |year: i32, month: time::Month| {
        let date = time::Date::from_calendar_date(year, month, 1).unwrap_or(local.date());
        let start = date.midnight().assume_offset(crate::time_fmt::prague_offset(date.midnight().assume_utc()));
        start.to_offset(time::UtcOffset::UTC).format(&time::format_description::well_known::Rfc3339).unwrap_or_default()
    };
    let this = month_start(local.year(), local.month());
    let (prev_year, prev_month) = if local.month() == time::Month::January { (local.year() - 1, time::Month::December) } else { (local.year(), local.month().previous()) };
    let previous = month_start(prev_year, prev_month);
    let this_month = sqlx::query_scalar!("SELECT COUNT(*) FROM orders WHERE created_at >= ?", this).fetch_one(db).await.map_err(err)?;
    let last_month = sqlx::query_scalar!("SELECT COUNT(*) FROM orders WHERE created_at >= ? AND created_at < ?", previous, this)
        .fetch_one(db)
        .await
        .map_err(err)?;
    Ok(Dashboard { products, orders, this_month, last_month, recent: order_rows(db, 5).await.map_err(err)? })
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <AdminLayout>
            <h1 class="admin-page-title">"Přehled"</h1>
            {loaded(load_dashboard(), |d: Dashboard| view! {
                <div class="admin-dashboard-cards">
                    <a href="/admin/catalog_products" class="admin-dashboard-card">
                        <span class="admin-dashboard-card-number">{d.products}</span>
                        <span class="admin-dashboard-card-label">"produktů v katalogu"</span>
                    </a>
                    <a href="/admin/orders" class="admin-dashboard-card">
                        <span class="admin-dashboard-card-number">{d.orders}</span>
                        <span class="admin-dashboard-card-label">"objednávek celkem"</span>
                    </a>
                    <a href="/admin/orders" class="admin-dashboard-card">
                        <span class="admin-dashboard-card-number">{d.this_month}</span>
                        <span class="admin-dashboard-card-label">"objednávky tento měsíc"</span>
                    </a>
                    <a href="/admin/orders" class="admin-dashboard-card">
                        <span class="admin-dashboard-card-number">{d.last_month}</span>
                        <span class="admin-dashboard-card-label">"objednávky minulý měsíc"</span>
                    </a>
                </div>
                <h2 class="admin-section-title">"Poslední objednávky"</h2>
                {if d.recent.is_empty() {
                    view! { <p>"Zatím žádné objednávky."</p> }.into_any()
                } else {
                    view! {
                        <table class="admin-table">
                            <thead><tr><th>"Č."</th><th>"Zákazník"</th><th>"Celkem"</th><th>"Datum"</th></tr></thead>
                            <tbody>
                                {d.recent.into_iter().map(|o| view! {
                                    <tr>
                                        <td><a href=format!("/admin/orders/{}", o.id)>{format!("#{}", o.id)}</a></td>
                                        <td>{o.name}</td>
                                        <td>{czk(o.total)}</td>
                                        <td>{when(&o.created_at)}</td>
                                    </tr>
                                }).collect_view()}
                            </tbody>
                        </table>
                        <a href="/admin/orders" class="admin-text-link">"Zobrazit všechny objednávky →"</a>
                    }.into_any()
                }}
            })}
        </AdminLayout>
    }
}

#[server(endpoint = "admin/orders")]
pub async fn load_orders() -> Result<Vec<OrderRow>, ServerFnError> {
    super::require_admin()?;
    let db = &expect_context::<crate::state::AppState>().db;
    order_rows(db, i64::MAX).await.map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn OrdersPage() -> impl IntoView {
    view! {
        <AdminLayout>
            <h1 class="admin-page-title">"Objednávky"</h1>
            {loaded(load_orders(), |orders: Vec<OrderRow>| if orders.is_empty() {
                view! { <p>"Zatím žádné objednávky."</p> }.into_any()
            } else {
                view! {
                    <table class="admin-table">
                        <thead><tr><th>"Č."</th><th>"Zákazník"</th><th>"E-mail"</th><th>"Celkem"</th><th>"Doprava"</th><th>"Platba"</th><th>"Datum"</th><th></th></tr></thead>
                        <tbody>
                            {orders.into_iter().map(|o| view! {
                                <tr>
                                    <td><a href=format!("/admin/orders/{}", o.id)>{format!("#{}", o.id)}</a></td>
                                    <td>{o.name}</td>
                                    <td>{o.email}</td>
                                    <td>{czk(o.total)}</td>
                                    <td>{if o.pickup { "Osobní odběr" } else { "Doručení" }}</td>
                                    <td>{if o.bank_transfer { "Převod" } else { "Hotově" }}</td>
                                    <td>{when(&o.created_at)}</td>
                                    <td><ConfirmButton action=format!("/admin/orders/{}/smazat", o.id) text="Smazat" confirm="Opravdu smazat tuto objednávku?"/></td>
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
pub struct OrderDetail {
    id: i64,
    created_at: String,
    name: String,
    email: String,
    phone: String,
    newsletter: bool,
    billing: [String; 3],
    company: Option<[String; 3]>,
    delivery: Option<[String; 3]>,
    pickup: bool,
    bank_transfer: bool,
    german: bool,
    /// (description, quantity, unit price, total)
    lines: Vec<(String, u32, String, String)>,
    totals: Vec<String>,
    total: String,
}

#[server(endpoint = "admin/order")]
pub async fn load_order(id: i64) -> Result<Option<OrderDetail>, ServerFnError> {
    use crate::i18n::{t, Locale};
    let v = super::require_admin()?;
    let db = &expect_context::<crate::state::AppState>().db;
    let Some(o) = sqlx::query!("SELECT * FROM orders WHERE id = ?", id).fetch_optional(db).await.map_err(|e| ServerFnError::new(e.to_string()))? else {
        return Ok(None);
    };
    let cs = Locale::Cs;
    let country = |c: &Option<String>| crate::orders::country_name(cs, c.as_deref());
    let lines: Vec<crate::orders::OrderLine> = serde_json::from_str(&o.items).unwrap_or_default();
    let lines = lines
        .iter()
        .map(|line| {
            let cart_line = line.as_cart_line();
            (v.shop.line_description(cs, &cart_line), line.quantity, czk(line.unit_price_czk), czk(cart_line.total().get()))
        })
        .collect();
    let mut totals = Vec::new();
    let discounted = o.discount_czk.is_some_and(|d| d > 0);
    let quoted = o.shipping_method == "address" && o.delivery_czk.is_some();
    if quoted || discounted {
        totals.push(format!("Zboží: {}", czk(o.subtotal_czk)));
    }
    if discounted {
        totals.push(format!("{}: −{}", t::order_mailer::discount(cs, o.discount_code.as_deref().unwrap_or_default()), czk(o.discount_czk.unwrap_or(0))));
    }
    if quoted {
        totals.push(format!("{}: {}", t::order_mailer::delivery(cs, o.delivery_distance_km.unwrap_or(0)), czk(o.delivery_czk.unwrap_or(0))));
    } else if o.shipping_method == "address" {
        totals.push("Doprava: vzdálenost se nepodařilo spočítat, cena dopravy není v celkové ceně.".into());
    }
    totals.push(format!("Bez DPH: {}", czk(o.total_czk - o.vat_czk)));
    let mut by_rate: Vec<(i64, i64)> = o
        .vat_breakdown
        .as_deref()
        .and_then(|j| serde_json::from_str::<std::collections::BTreeMap<String, i64>>(j).ok())
        .map(|m| m.into_iter().filter_map(|(r, a)| Some((r.parse().ok()?, a))).collect())
        .unwrap_or_else(|| vec![(21, o.vat_czk)]);
    by_rate.sort_by(|a, b| b.0.cmp(&a.0));
    let vat_country = country(&Some(o.vat_country.clone().unwrap_or_else(|| "cz".into())));
    for (rate, amount) in &by_rate {
        totals.push(format!("DPH {rate} % ({vat_country}): {}", czk(*amount)));
    }
    if by_rate.len() > 1 {
        totals.push(format!("DPH celkem: {}", czk(o.vat_czk)));
    }
    let opt = |s: Option<String>| s.unwrap_or_default();
    Ok(Some(OrderDetail {
        id: o.id,
        created_at: o.created_at,
        name: format!("{} {}", o.first_name, o.last_name).trim().to_owned(),
        email: o.email,
        phone: o.phone,
        newsletter: o.newsletter_opt_in != 0,
        billing: [o.billing_street, format!("{} {}", o.billing_zip, o.billing_city), country(&Some(o.billing_country))],
        company: (o.company_purchase != 0).then(|| [opt(o.company_name), opt(o.company_ico), opt(o.company_dic)]),
        delivery: (o.delivery_address_different != 0).then(|| {
            [opt(o.delivery_street), format!("{} {}", opt(o.delivery_zip), opt(o.delivery_city)), country(&o.delivery_country)]
        }),
        pickup: o.shipping_method == "pickup",
        bank_transfer: o.payment_method == "bank_transfer",
        german: o.locale == "de",
        lines,
        totals,
        total: czk(o.total_czk),
    }))
}

fn lines3(lines: [String; 3]) -> impl IntoView {
    let [a, b, c] = lines;
    view! { {a}<br/>{b}<br/>{c} }
}

#[component]
pub fn OrderPage() -> impl IntoView {
    let id = use_params_map().with_untracked(|p| p.get("id").and_then(|id| id.parse().ok()).unwrap_or(0));
    view! {
        <AdminLayout>
            {loaded(load_order(id), |o: Option<OrderDetail>| match o {
                None => view! { <p>"Objednávka nenalezena."</p> }.into_any(),
                Some(o) => view! {
                    <a href="/admin/orders" class="admin-back-link">"← Zpět na objednávky"</a>
                    <div class="admin-toolbar">
                        <div>
                            <h1 class="admin-page-title">{format!("Objednávka #{}", o.id)}</h1>
                            <p class="admin-page-subtitle">{when(&o.created_at)}</p>
                        </div>
                        <ConfirmButton action=format!("/admin/orders/{}/smazat", o.id) text="Smazat objednávku" confirm="Opravdu smazat tuto objednávku?"/>
                    </div>
                    <div class="admin-detail-grid">
                        <div class="admin-box">
                            <h2>"Zákazník"</h2>
                            <p>{o.name}<br/><a href=format!("mailto:{}", o.email)>{o.email.clone()}</a><br/>{o.phone}</p>
                            <p>"Newsletter: "{if o.newsletter { "Ano" } else { "Ne" }}</p>
                        </div>
                        <div class="admin-box">
                            <h2>"Fakturační adresa"</h2>
                            <p>{lines3(o.billing)}</p>
                            {o.company.map(|[name, ico, dic]| view! { <p><strong>{name}</strong><br/>"IČO: "{ico}<br/>"DIČ: "{dic}</p> })}
                        </div>
                        {o.delivery.map(|d| view! { <div class="admin-box"><h2>"Doručovací adresa"</h2><p>{lines3(d)}</p></div> })}
                        <div class="admin-box">
                            <h2>"Doprava a platba"</h2>
                            <p>
                                {if o.pickup { "Osobní odběr na adrese skladu" } else { "Doručení na adresu" }}<br/>
                                {if o.bank_transfer { "Bankovním převodem - proforma" } else { "Hotově při vyzvednutí" }}
                            </p>
                            <p>"Jazyk zákazníka: "{if o.german { "němčina" } else { "čeština" }}</p>
                        </div>
                    </div>
                    <div class="admin-box">
                        <h2>"Obsah objednávky"</h2>
                        <table class="admin-table">
                            <thead><tr><th>"Položka"</th><th>"Množství"</th><th>"Jedn. cena"</th><th>"Celkem"</th></tr></thead>
                            <tbody>
                                {o.lines.into_iter().map(|(d, q, u, t)| view! { <tr><td>{d}</td><td>{q}</td><td>{u}</td><td>{t}</td></tr> }).collect_view()}
                            </tbody>
                        </table>
                        <div class="admin-order-totals">
                            {o.totals.into_iter().map(|t| view! { <p>{t}</p> }).collect_view()}
                            <p><strong>"Celkem k úhradě: "{o.total}</strong></p>
                        </div>
                    </div>
                }.into_any(),
            })}
        </AdminLayout>
    }
}

/// Inquiry categories and fields as the admin names them.
#[cfg(feature = "ssr")]
fn category_label(category: &str) -> &'static str {
    if category == "palivove" { "Palivové dřevo" } else { "Stavební řezivo" }
}

pub fn field_label(field: &str) -> &'static str {
    match field {
        "varianta" => "Varianta",
        "druh" => "Druh",
        "delka" => "Délka",
        "mnozstvi" => "Množství",
        "polozka" => "Položka",
        "vyska" => "Výška",
        _ => "Šířka",
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InquiryRow {
    id: i64,
    name: String,
    email: String,
    phone: String,
    categories: String,
    items: usize,
    created_at: String,
}

#[server(endpoint = "admin/inquiries")]
pub async fn load_inquiries() -> Result<Vec<InquiryRow>, ServerFnError> {
    super::require_admin()?;
    let db = &expect_context::<crate::state::AppState>().db;
    Ok(sqlx::query!("SELECT id, first_name, last_name, email, phone, items, created_at FROM inquiries ORDER BY created_at DESC")
        .fetch_all(db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .into_iter()
        .map(|i| {
            let items: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&i.items).unwrap_or_default();
            let categories: Vec<&str> = crate::pages::kontakt::ROW_FIELDS.iter().map(|(c, _)| *c).filter(|c| items.contains_key(*c)).collect();
            InquiryRow {
                id: i.id,
                name: format!("{} {}", i.first_name, i.last_name).trim().to_owned(),
                email: i.email,
                phone: i.phone,
                categories: categories.iter().map(|c| category_label(c)).collect::<Vec<_>>().join(" + "),
                items: items.values().filter_map(|v| v.as_array()).map(Vec::len).sum(),
                created_at: i.created_at,
            }
        })
        .collect())
}

#[component]
pub fn InquiriesPage() -> impl IntoView {
    view! {
        <AdminLayout>
            <h1 class="admin-page-title">"Poptávky"</h1>
            {loaded(load_inquiries(), |rows: Vec<InquiryRow>| if rows.is_empty() {
                view! { <div class="admin-empty-state">"Zatím žádné poptávky."</div> }.into_any()
            } else {
                view! {
                    <table class="admin-table">
                        <thead><tr><th>"Č."</th><th>"Jméno"</th><th>"E-mail"</th><th>"Telefon"</th><th>"Kategorie"</th><th>"Položek"</th><th>"Datum"</th></tr></thead>
                        <tbody>
                            {rows.into_iter().map(|i| view! {
                                <tr>
                                    <td><a href=format!("/admin/inquiries/{}", i.id)>{format!("#{}", i.id)}</a></td>
                                    <td>{i.name}</td>
                                    <td>{i.email}</td>
                                    <td>{i.phone}</td>
                                    <td>{i.categories}</td>
                                    <td>{i.items}</td>
                                    <td>{when(&i.created_at)}</td>
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
pub struct InquiryDetail {
    id: i64,
    created_at: String,
    german: bool,
    name: String,
    email: String,
    phone: String,
    address: String,
    /// (category label, field labels, rows of values)
    categories: Vec<(String, Vec<String>, Vec<Vec<String>>)>,
    notes: Option<String>,
}

#[server(endpoint = "admin/inquiry")]
pub async fn load_inquiry(id: i64) -> Result<Option<InquiryDetail>, ServerFnError> {
    super::require_admin()?;
    let db = &expect_context::<crate::state::AppState>().db;
    let Some(i) = sqlx::query!("SELECT * FROM inquiries WHERE id = ?", id).fetch_optional(db).await.map_err(|e| ServerFnError::new(e.to_string()))? else {
        return Ok(None);
    };
    let items: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&i.items).unwrap_or_default();
    let categories = crate::pages::kontakt::ROW_FIELDS
        .iter()
        .filter_map(|(category, fields)| {
            let rows = items.get(*category)?.as_array()?;
            Some((
                category_label(category).to_owned(),
                fields.iter().map(|f| field_label(f).to_owned()).collect(),
                rows.iter().map(|row| fields.iter().map(|f| row.get(*f).and_then(|v| v.as_str()).unwrap_or_default().to_owned()).collect()).collect(),
            ))
        })
        .collect();
    let join = |parts: [Option<&str>; 2]| parts.into_iter().flatten().filter(|p| !p.is_empty()).collect::<Vec<_>>().join(" ");
    let address = [join([i.street.as_deref(), i.house_number.as_deref()]), join([i.zip.as_deref(), i.city.as_deref()])]
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(", ");
    Ok(Some(InquiryDetail {
        id: i.id,
        created_at: i.created_at,
        german: i.locale == "de",
        name: format!("{} {}", i.first_name, i.last_name).trim().to_owned(),
        email: i.email,
        phone: i.phone,
        address,
        categories,
        notes: i.notes.filter(|n| !n.trim().is_empty()),
    }))
}

#[component]
pub fn InquiryPage() -> impl IntoView {
    let id = use_params_map().with_untracked(|p| p.get("id").and_then(|id| id.parse().ok()).unwrap_or(0));
    view! {
        <AdminLayout>
            {loaded(load_inquiry(id), |i: Option<InquiryDetail>| match i {
                None => view! { <p>"Poptávka nenalezena."</p> }.into_any(),
                Some(i) => view! {
                    <a href="/admin/inquiries" class="admin-back-link">"← Zpět na poptávky"</a>
                    <h1 class="admin-page-title">{format!("Poptávka #{}", i.id)}</h1>
                    <p class="admin-page-subtitle">{when(&i.created_at)}" · jazyk formuláře: "{if i.german { "němčina" } else { "čeština" }}</p>
                    <div class="admin-detail-grid">
                        <div class="admin-box">
                            <h2>"Kontakt"</h2>
                            <p>{i.name}<br/><a href=format!("mailto:{}", i.email)>{i.email.clone()}</a><br/>{i.phone}</p>
                            {(!i.address.is_empty()).then(|| view! { <p>{i.address}</p> })}
                        </div>
                    </div>
                    <div class="admin-box">
                        {i.categories.into_iter().enumerate().map(|(n, (label, fields, rows))| view! {
                            <h2 class={if n > 0 { "admin-inquiry-next-category" } else { "" }}>"Požadované položky – "{label}</h2>
                            <table class="admin-table">
                                <thead><tr>{fields.into_iter().map(|f| view! { <th>{f}</th> }).collect_view()}</tr></thead>
                                <tbody>
                                    {rows.into_iter().map(|row| view! { <tr>{row.into_iter().map(|c| view! { <td>{c}</td> }).collect_view()}</tr> }).collect_view()}
                                </tbody>
                            </table>
                        }).collect_view()}
                        {i.notes.map(|n| view! {
                            <h2 style="margin-top: 20px;">"Poznámka"</h2>
                            <p style="white-space: pre-wrap;">{n}</p>
                        })}
                    </div>
                }.into_any(),
            })}
        </AdminLayout>
    }
}

#[cfg(feature = "ssr")]
pub mod handlers {
    use super::super::handlers::{done, guard};
    use crate::state::AppState;
    use crate::visitor::Visitor;
    use axum::extract::{Path, State};
    use axum::response::Response;
    use axum::Extension;
    use tower_sessions::Session;

    pub async fn delete(State(state): State<AppState>, session: Session, Extension(v): Extension<Visitor>, Path(id): Path<i64>) -> Response {
        if let Err(r) = guard(&session, &v).await {
            return r;
        }
        let _ = sqlx::query!("DELETE FROM orders WHERE id = ?", id).execute(&state.db).await;
        done(&session, "/admin/orders", "Objednávka byla smazána.").await
    }
}
