//! Checkout steps 2–4 (shipping and payment, details, summary) and the
//! thank-you page.

use super::cart::CheckoutStepper;
use super::{PageMeta, loaded};
use crate::cart_totals::TotalRows;
use crate::cart_view::{LineView, TotalRow};
use crate::i18n::t;
use crate::layout::Layout;
use crate::visitor::{Visitor, visitor};
use domain::checkout::{Payment, Shipping};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Where a step sends the visitor when an earlier one isn't done yet.
fn step_redirect(v: &Visitor, step: usize) -> Option<&'static str> {
    if v.cart.is_empty() {
        Some("/kosik")
    } else if step >= 3 && !v.checkout.shipping_selected() {
        Some("/kosik/doprava")
    } else if step >= 4 && !v.checkout.details_complete() {
        Some("/kosik/udaje")
    } else {
        None
    }
}

/// Sends the browser to `path` (in the visitor's language).
#[allow(unused_variables)]
fn redirect(v: &Visitor, path: &str) {
    #[cfg(feature = "ssr")]
    leptos_axum::redirect(&v.link(path));
}

/// Private pages aren't cached anywhere.
fn no_store() {
    #[cfg(feature = "ssr")]
    if let Some(response) = use_context::<leptos_axum::ResponseOptions>() {
        response.insert_header(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-store"),
        );
    }
}

/// The cart's lines and totals beside the steps (without the estimate).
#[component]
fn CheckoutSidebar(v: Visitor) -> impl IntoView {
    let l = v.locale;
    #[cfg(feature = "ssr")]
    let (lines, totals) = (
        crate::cart_view::lines(&v),
        crate::cart_view::totals_with(&v, false, false),
    );
    #[cfg(not(feature = "ssr"))]
    let (lines, totals): (Vec<LineView>, Vec<TotalRow>) = (vec![], vec![]);
    view! {
        <aside class="checkout-summary-box">
            <h3 class="checkout-summary-title">{t::checkout::order_contents(l)}</h3>
            <ul class="checkout-summary-items">
                {lines.into_iter().map(|line| view! {
                    <li class="checkout-summary-item">
                        {line.image.clone().map(|src| view! { <img src=src alt=line.description.clone() class="checkout-summary-item-img"/> })}
                        <span class="checkout-summary-item-desc">{line.description.clone()}</span>
                    </li>
                }).collect_view()}
            </ul>
            <div class="cart-totals checkout-summary-totals"><TotalRows rows=totals/></div>
        </aside>
    }
}

fn radio(
    name: &'static str,
    value: &'static str,
    checked: bool,
    label: &'static str,
    note: String,
) -> impl IntoView {
    view! {
        <label class="checkout-radio-option">
            <span>
                <input type="radio" name=name value=value checked=checked/>
                " "{label}
            </span>
            " "
            <span class="checkout-radio-note">{note}</span>
        </label>
    }
}

fn country_options(l: crate::i18n::Locale, selected: &str) -> impl IntoView + use<> {
    view! {
        <option value="cz" selected=selected == "cz">{t::checkout::country_cz(l)}</option>
        <option value="at" selected=selected == "at">{t::checkout::country_at(l)}</option>
    }
}

#[component]
pub fn ShippingPage() -> impl IntoView {
    no_store();
    let v = visitor();
    let l = v.locale;
    if let Some(path) = step_redirect(&v, 2) {
        redirect(&v, path);
    }
    let c = v.checkout.clone();
    let country = c
        .country
        .clone()
        .or_else(|| v.delivery.as_ref().map(|d| d.country.as_str().to_owned()))
        .unwrap_or_else(|| "cz".into());
    let address_note = match &v.delivery {
        Some(d) => t::delivery_estimate::shipping_note(
            l,
            &d.place,
            v.shop.bare_price(l, d.quote(v.shop.czk_per_km).price()),
        ),
        None => t::checkout::shipping_address_rate_note(
            l,
            v.shop.price(l, domain::Czk(v.shop.czk_per_km)),
        ),
    };
    let free = t::checkout::free(l).to_owned();
    let body = view! {
            <main class="main">
                <section class="checkout-section">
                    <CheckoutStepper current=2/>
                    <div class="checkout-layout">
                        <form action={v.link("/kosik/doprava")} method="post" class="checkout-main-col">
                            <div class="checkout-box">
                                <h3 class="checkout-box-title">{t::checkout::delivery_country_label(l)}</h3>
                                <div class="checkout-box-body">
                                    <label for="country">{t::checkout::country_field(l)}</label>
                                    <select name="country" id="country">{country_options(l, &country)}</select>
                                </div>
                            </div>
                            <div class="checkout-box">
                                <h3 class="checkout-box-title">{t::checkout::shipping_method_label(l)}</h3>
                                <div class="checkout-box-body checkout-radio-list">
                                    {radio("shipping_method", "address", c.shipping == Some(Shipping::Address), t::checkout::shipping_address(l), address_note)}
                                    {radio("shipping_method", "pickup", c.shipping != Some(Shipping::Address), t::checkout::shipping_pickup(l), free.clone())}
                                </div>
                            </div>
                            <div class="checkout-box">
                                <h3 class="checkout-box-title">{t::checkout::payment_method_label(l)}</h3>
                                <div class="checkout-box-body checkout-radio-list">
                                    {radio("payment_method", "cash", c.payment != Some(Payment::BankTransfer), t::checkout::payment_cash(l), free.clone())}
                                    {radio("payment_method", "bank_transfer", c.payment == Some(Payment::BankTransfer), t::checkout::payment_bank_transfer(l), free)}
                                </div>
                            </div>
                            <div class="checkout-nav">
                                <a href=v.link("/kosik") class="btn-outline-dark">{t::checkout::back(l)}</a>
                                <input type="submit" value=t::checkout::r#continue(l) class="btn-primary"/>
                            </div>
                        </form>
                        <CheckoutSidebar v=v.clone()/>
                    </div>
                </section>
            </main>
    };
    view! {
        <PageMeta title=t::meta::checkout_shipping::title(l).to_owned()/>
        <Layout>{body}</Layout>
    }
}

#[component]
pub fn DetailsPage() -> impl IntoView {
    no_store();
    let v = visitor();
    let l = v.locale;
    if let Some(path) = step_redirect(&v, 3) {
        redirect(&v, path);
    }
    // Filled in from the account (first time) and from "Kam vozíme?".
    let mut c = v.checkout.clone();
    if c.first_name.is_none()
        && let Some(profile) = v.user.as_ref().map(|u| u.profile.clone())
    {
        c.first_name = profile.first_name;
        c.last_name = profile.last_name;
        c.phone = profile.phone;
        c.email = v.user.as_ref().map(|u| u.email.clone());
        c.billing = profile.billing;
        c.company_purchase = profile.company_purchase;
        c.company_name = profile.company_name;
        c.company_ico = profile.company_ico;
        c.company_dic = profile.company_dic;
        c.delivery_address_different = profile.delivery_address_different;
        c.delivery = profile.delivery;
    }
    if c.billing.city.is_none()
        && let Some(d) = &v.delivery
    {
        c.billing.city = Some(d.place.clone());
        c.billing.zip = d.zip.clone();
    }
    let val = |o: &Option<String>| o.clone().unwrap_or_default();
    let billing_country = c
        .billing
        .country
        .clone()
        .or_else(|| c.country.clone())
        .unwrap_or_else(|| "cz".into());
    let delivery_country = c.delivery.country.clone().unwrap_or_else(|| "cz".into());
    let field = move |name: &'static str,
                      label: &'static str,
                      kind: &'static str,
                      value: String,
                      required: bool,
                      wide: bool| {
        view! {
            <div class=if wide { "form-group form-group-wide" } else { "form-group" }>
                <label for=name>{label}</label>
                <input type=kind name=name id=name value=value required=required/>
            </div>
        }
    };
    let street = move |name: &'static str,
                       value: String,
                       required: bool,
                       city: &'static str,
                       zip: &'static str,
                       country: &'static str| {
        view! {
            <div class="form-group form-group-wide address-autocomplete">
                <label for=name>{t::checkout::street_and_number(l)}</label>
                <crate::autocomplete::AutocompleteInput
                    id=name.to_owned()
                    name=name.to_owned()
                    value=value
                    placeholder=String::new()
                    address=true
                    required=required
                    country_select=country.to_owned()
                    city_field=city.to_owned()
                    zip_field=zip.to_owned()
                    submit_on_pick=false
                />
            </div>
        }
    };
    let checkbox = move |name: &'static str, checked: bool, label: &'static str| {
        view! {
            <label class="custom-checkbox-label checkout-checkbox">
                <input type="hidden" name=name value="0"/>
                <input type="checkbox" name=name value="1" checked=checked/>
                <span class="custom-checkbox"></span>
                <span>{label}</span>
            </label>
        }
    };
    let login = match &v.user {
        Some(user) => view! { <p class="checkout-logged-in-as">{t::checkout::logged_in_as(l, &user.email)}</p> }.into_any(),
        None => view! {
            <a href=v.link(&format!("/prihlaseni?return_to={}", v.link("/kosik/udaje"))) class="btn-outline-dark checkout-login-btn">{t::checkout::login_existing_user(l)}</a>
        }
        .into_any(),
    };
    let body = view! {
            <main class="main">
                <section class="checkout-section">
                    <CheckoutStepper current=3/>
                    <div class="checkout-layout">
                        <form action={v.link("/kosik/udaje")} method="post" class="checkout-main-col">
                            <div class="checkout-box">
                                <h3 class="checkout-box-title">{t::checkout::personal_details(l)}</h3>
                                <div class="checkout-box-body">
                                    {login}
                                    <div class="checkout-form-grid">
                                        {field("first_name", t::kontakt::form_jmeno(l), "text", val(&c.first_name), true, false)}
                                        {field("last_name", t::kontakt::form_prijmeni(l), "text", val(&c.last_name), true, false)}
                                        {field("email", t::kontakt::form_email(l), "email", val(&c.email), true, false)}
                                        {field("phone", t::kontakt::form_telefon(l), "tel", val(&c.phone), true, false)}
                                    </div>
                                    {checkbox("newsletter_opt_in", c.newsletter_opt_in, t::checkout::newsletter_opt_in(l))}
                                </div>
                            </div>
                            <div class="checkout-box">
                                <h3 class="checkout-box-title">{t::checkout::billing_details(l)}</h3>
                                <div class="checkout-box-body">
                                    <div class="checkout-form-grid">
                                        {street("billing_street", val(&c.billing.street), true, "billing_city", "billing_zip", "billing_country")}
                                        {field("billing_city", t::kontakt::form_mesto(l), "text", val(&c.billing.city), true, false)}
                                        {field("billing_zip", t::kontakt::form_psc(l), "text", val(&c.billing.zip), true, false)}
                                        <div class="form-group">
                                            <label for="billing_country">{t::checkout::country_field(l)}</label>
                                            <select name="billing_country" id="billing_country">{country_options(l, &billing_country)}</select>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <details class="checkout-box checkout-collapsible" open=c.company_purchase>
                                <summary class="checkout-box-title">{t::checkout::company_purchase(l)}</summary>
                                <div class="checkout-box-body">
                                    {checkbox("company_purchase", c.company_purchase, t::checkout::company_purchase(l))}
                                    <div class="checkout-form-grid">
                                        {field("company_name", t::checkout::company_name(l), "text", val(&c.company_name), false, true)}
                                        {field("company_ico", t::checkout::company_ico(l), "text", val(&c.company_ico), false, false)}
                                        {field("company_dic", t::checkout::company_dic(l), "text", val(&c.company_dic), false, false)}
                                    </div>
                                </div>
                            </details>
                            <details class="checkout-box checkout-collapsible" open=c.delivery_address_different>
                                <summary class="checkout-box-title">{t::checkout::delivery_address_different(l)}</summary>
                                <div class="checkout-box-body">
                                    {checkbox("delivery_address_different", c.delivery_address_different, t::checkout::delivery_address_different(l))}
                                    <div class="checkout-form-grid">
                                        {street("delivery_street", val(&c.delivery.street), false, "delivery_city", "delivery_zip", "delivery_country")}
                                        {field("delivery_city", t::kontakt::form_mesto(l), "text", val(&c.delivery.city), false, false)}
                                        {field("delivery_zip", t::kontakt::form_psc(l), "text", val(&c.delivery.zip), false, false)}
                                        <div class="form-group">
                                            <label for="delivery_country">{t::checkout::country_field(l)}</label>
                                            <select name="delivery_country" id="delivery_country">{country_options(l, &delivery_country)}</select>
                                        </div>
                                    </div>
                                </div>
                            </details>
                            <div class="checkout-nav">
                                <a href=v.link("/kosik/doprava") class="btn-outline-dark">{t::checkout::back(l)}</a>
                                <input type="submit" value=t::checkout::r#continue(l) class="btn-primary"/>
                            </div>
                        </form>
                        <CheckoutSidebar v=v.clone()/>
                    </div>
                </section>
            </main>
    };
    view! {
        <PageMeta title=t::meta::checkout_details::title(l).to_owned()/>
        <Layout>{body}</Layout>
    }
}

/// The summary step's figures (worked out on the server, with the road
/// distance to the address).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Summary {
    lines: Vec<LineView>,
    shipping_note: String,
    shipping_note_strong: Option<String>,
    rows: Vec<TotalRow>,
    discount_problem: Option<String>,
    billing: Vec<Vec<String>>,
}

#[server(endpoint = "page/checkout-summary")]
pub async fn load_summary() -> Result<Summary, ServerFnError> {
    use crate::cart_view::{RowKind, TotalRow as Row};
    let v = visitor();
    let l = v.locale;
    let state = expect_context::<crate::state::AppState>();
    let d = crate::orders::draft(&state, &v).await;
    let price = |a: domain::Czk| v.shop.price(l, a);
    let lines = d
        .lines
        .iter()
        .map(|line| {
            let cart_line = line.as_cart_line();
            LineView {
                variant_id: line.catalog_variant_id,
                image: line.image.clone(),
                description: v.shop.line_description(l, &cart_line),
                quantity: line.quantity,
                unit_price: price(cart_line.unit_price),
                total: price(cart_line.total()),
                total_excl_vat: price(v.shop.vat.excl_vat(
                    line.category,
                    cart_line.total(),
                    d.vat_country,
                )),
            }
        })
        .collect();
    let (shipping_note, shipping_note_strong) = match (v.checkout.shipping, d.delivery) {
        (Some(Shipping::Address), Some(q)) => (
            t::checkout::delivery_approximate(l).to_owned(),
            Some(price(q.price())),
        ),
        (Some(Shipping::Address), None) => (t::checkout::shipping_address_note(l).to_owned(), None),
        _ => (t::checkout::free(l).to_owned(), None),
    };
    let mut rows = Vec::new();
    if d.delivery.is_some() || d.discount.is_some() {
        rows.push(Row {
            label: t::checkout::goods_total(l).into(),
            value: price(d.subtotal),
            kind: RowKind::Muted,
        });
    }
    if let Some((code, amount)) = &d.discount {
        rows.push(Row {
            label: t::cart::discount_line(l, code),
            value: format!("−{}", price(*amount)),
            kind: RowKind::Discount,
        });
    }
    if let Some(q) = d.delivery {
        rows.push(Row {
            label: t::checkout::delivery_line(l, q.round_trip_km()),
            value: price(q.price()),
            kind: RowKind::Muted,
        });
    }
    rows.push(Row {
        label: t::cart::total_due(l).into(),
        value: price(d.total),
        kind: RowKind::Total,
    });
    rows.extend(crate::cart_view::vat_rows(&v, &d.vat.by_rate));
    rows.push(Row {
        label: t::cart::without_vat(l).into(),
        value: price(d.total - d.vat.vat()),
        kind: RowKind::Muted,
    });

    let c = &v.checkout;
    let o = |s: &Option<String>| s.clone().unwrap_or_default();
    let mut billing = vec![
        vec![
            format!("{} {}", o(&c.first_name), o(&c.last_name))
                .trim()
                .to_owned(),
            o(&c.email),
            format!("{}: {}", t::kontakt::form_telefon(l), o(&c.phone)),
        ],
        vec![
            o(&c.billing.street),
            format!("{} {}", o(&c.billing.zip), o(&c.billing.city)),
            crate::orders::country_name(l, c.billing.country.as_deref()),
        ],
    ];
    if c.company_purchase {
        billing.push(vec![
            o(&c.company_name),
            format!("{}: {}", t::checkout::company_ico(l), o(&c.company_ico)),
            format!("{}: {}", t::checkout::company_dic(l), o(&c.company_dic)),
        ]);
    }
    Ok(Summary {
        lines,
        shipping_note,
        shipping_note_strong,
        rows,
        discount_problem: d.discount_problem,
        billing,
    })
}

#[component]
pub fn SummaryPage() -> impl IntoView {
    no_store();
    let v = visitor();
    let l = v.locale;
    let redirecting = step_redirect(&v, 4);
    if let Some(path) = redirecting {
        redirect(&v, path);
    }
    let errors = v.checkout_errors.clone();
    let shipping = if v.checkout.shipping == Some(Shipping::Address) {
        t::checkout::shipping_address(l)
    } else {
        t::checkout::shipping_pickup(l)
    };
    let payment = if v.checkout.payment == Some(Payment::BankTransfer) {
        t::checkout::payment_bank_transfer(l)
    } else {
        t::checkout::payment_cash(l)
    };
    let link = |path: &str, text: &str| {
        format!(
            r#"<a target="_blank" rel="noopener" href="{}">{}</a>"#,
            v.link(path),
            codon::Html::escape(text)
        )
    };
    let terms = t::checkout::terms_agreement(
        l,
        codon::Html::from_trusted(link(
            "/zasady-ochrany-osobnich-udaju",
            t::checkout::privacy_link_text(l),
        )),
        codon::Html::from_trusted(link("/obchodni-podminky", t::checkout::terms_link_text(l))),
    )
    .into_string();
    let has_errors = !errors.is_empty();
    view! {
        <PageMeta title=t::meta::checkout_summary::title(l).to_owned()/>
        <Layout>
            <main class="main">
                <section class="checkout-section">
                    <CheckoutStepper current=4/>
                    {has_errors.then(|| view! {
                        <div class="form-error">
                            <p>{t::checkout::submit_error(l)}</p>
                            <ul>{errors.iter().map(|e| view! { <li>{e.clone()}</li> }).collect_view()}</ul>
                        </div>
                    })}
                    {redirecting.is_none().then(|| loaded(load_summary(), move |s: Summary| {
                        let v = visitor();
                        view! {
                            <div class="checkout-layout">
                                <form action={v.link("/kosik/souhrn")} method="post" class="checkout-main-col">
                                    <div class="checkout-box">
                                        <h3 class="checkout-box-title">{t::checkout::order_contents(l)}</h3>
                                        <div class="checkout-box-body">
                                            <ul class="checkout-summary-full-items">
                                                {s.lines.into_iter().map(|line| view! {
                                                    <li class="checkout-summary-full-item">
                                                        {line.image.clone().map(|src| view! { <img src=src alt=line.description.clone() class="cart-item-img"/> })}
                                                        <span class="cart-item-desc">{line.description.clone()}</span>
                                                        <div class="checkout-summary-full-item-prices">
                                                            <div class="cart-total-row cart-total-excl-vat">
                                                                <span>{t::cart::table_price_excl_vat(l)}</span>
                                                                <span>{line.total_excl_vat}</span>
                                                            </div>
                                                            <div class="cart-total-row">
                                                                <span>{t::cart::table_price_incl_vat(l)}</span>
                                                                <strong>{line.total}</strong>
                                                            </div>
                                                        </div>
                                                    </li>
                                                }).collect_view()}
                                            </ul>
                                        </div>
                                    </div>
                                    <div class="checkout-box checkout-method-summary">
                                        <div class="checkout-box-body checkout-radio-list">
                                            <div class="checkout-radio-option checkout-radio-option-readonly">
                                                <span><span class="checkout-radio-dot"></span>" "{shipping}</span>
                                                " "
                                                <span class="checkout-radio-note">
                                                    {s.shipping_note_strong.map(|p| view! { <strong>{p}</strong>" " })}
                                                    {s.shipping_note}
                                                </span>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="checkout-box checkout-method-summary">
                                        <div class="checkout-box-body checkout-radio-list">
                                            <div class="checkout-radio-option checkout-radio-option-readonly">
                                                <span><span class="checkout-radio-dot"></span>" "{payment}</span>
                                                " "
                                                <span class="checkout-radio-note">{t::checkout::free(l)}</span>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="checkout-box">
                                        <div class="checkout-box-body">
                                            {s.discount_problem.filter(|_| !has_errors).map(|p| view! {
                                                <p class="discount-code-message discount-code-message-error">{p}</p>
                                            })}
                                            <TotalRows rows=s.rows/>
                                        </div>
                                    </div>
                                    <p class="pay-later-note">{t::checkout::pay_later_note(l)}</p>
                                    <crate::islands::TermsAgreement label_html=terms.clone() error=t::checkout::terms_required(l).to_owned()/>
                                    <div class="checkout-nav">
                                        <a href=v.link("/kosik/udaje") class="btn-outline-dark">{t::checkout::back(l)}</a>
                                        <input type="submit" value=t::checkout::r#continue(l) class="btn-primary"/>
                                    </div>
                                </form>
                                <aside class="checkout-summary-box">
                                    <h3 class="checkout-summary-title">{t::checkout::billing_details(l)}</h3>
                                    {s.billing.into_iter().enumerate().map(|(i, block)| {
                                        let email = (i == 0).then(|| block.get(1).cloned()).flatten();
                                        view! {
                                            <p class="checkout-billing-summary">
                                                {block.into_iter().enumerate().map(|(j, line)| {
                                                    let shown = match (&email, j) {
                                                        (Some(e), 1) => view! { <a href=format!("mailto:{e}")>{e.clone()}</a> }.into_any(),
                                                        _ => view! { {line} }.into_any(),
                                                    };
                                                    view! { {(j > 0).then(|| view! { <br/> })}{shown} }
                                                }).collect_view()}
                                            </p>
                                        }
                                    }).collect_view()}
                                </aside>
                            </div>
                        }
                    }))}
                </section>
            </main>
        </Layout>
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Confirmation {
    id: i64,
    pay_later: String,
    /// The Google Analytics purchase event's data (JSON).
    purchase: String,
}

#[server(endpoint = "page/checkout-confirmation")]
pub async fn load_confirmation() -> Result<Option<Confirmation>, ServerFnError> {
    let v = visitor();
    let Some(id) = v.last_order else {
        return Ok(None);
    };
    let db = &expect_context::<crate::state::AppState>().db;
    let Some(order) = sqlx::query!(
        "SELECT id, items, total_czk, payment_method FROM orders WHERE id = ?",
        id
    )
    .fetch_optional(db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    else {
        return Ok(None);
    };
    let lines: Vec<crate::orders::OrderLine> =
        serde_json::from_str(&order.items).unwrap_or_default();
    let items: Vec<serde_json::Value> = lines
        .iter()
        .map(|line| {
            serde_json::json!({
                "item_id": line.product_key,
                "item_name": v.shop.product_title(crate::i18n::Locale::Cs, &line.product_key),
                "price": line.unit_price_czk as f64,
                "quantity": line.quantity,
            })
        })
        .collect();
    let purchase = serde_json::json!({
        "transaction_id": order.id.to_string(),
        "value": order.total_czk as f64,
        "currency": "CZK",
        "items": items,
    });
    Ok(Some(Confirmation {
        id: order.id,
        pay_later: crate::orders::pay_later_text(v.locale, Payment::parse(&order.payment_method))
            .to_owned(),
        purchase: purchase.to_string().replace('<', "\\u003c"),
    }))
}

#[component]
pub fn ConfirmationPage() -> impl IntoView {
    no_store();
    let v = visitor();
    let l = v.locale;
    view! {
        <PageMeta title=t::meta::checkout_confirmation::title(l).to_owned()/>
        <Layout>
            {loaded(load_confirmation(), move |order: Option<Confirmation>| {
                let v = visitor();
                match order {
                    None => {
                        redirect(&v, "/kosik");
                        ().into_any()
                    }
                    Some(order) => {
                        let nonce = leptos::nonce::use_nonce();
                        // Sent once per order, even if the page is reloaded.
                        let script = format!(
                            r#"(function () {{
  var dedupeKey = "ga_purchase_fired_{id}";
  try {{
    if (sessionStorage.getItem(dedupeKey)) return;
    sessionStorage.setItem(dedupeKey, "1");
  }} catch (e) {{}}
  if (typeof gtag === "function") gtag("event", "purchase", {data});
}})();"#,
                            id = order.id,
                            data = order.purchase
                        );
                        view! {
                            <main class="main">
                                <section class="checkout-section checkout-confirmation">
                                    <div class="checkout-confirmation-box">
                                        <svg viewBox="0 0 24 24" width="56" height="56" fill="none" stroke="#2C8FFF" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <circle cx="12" cy="12" r="10"></circle>
                                            <polyline points="8 12 11 15 16 9"></polyline>
                                        </svg>
                                        <h1>{t::checkout::confirmation_title(l)}</h1>
                                        <p>{t::checkout::confirmation_text(l, order.id)}</p>
                                        <div class="pay-later-notice" role="note">
                                            <strong>{t::checkout::pay_later_title(l)}</strong>
                                            <p>{order.pay_later}</p>
                                        </div>
                                        <a href=v.link("/") class="btn-primary">{t::checkout::back_to_homepage(l)}</a>
                                    </div>
                                    <img src="/img/mascot/thumbs-up.webp" alt="" class="checkout-confirmation-mascot" width="320" height="320" loading="eager"/>
                                </section>
                                <script nonce=nonce>{script}</script>
                            </main>
                        }
                        .into_any()
                    }
                }
            })}
        </Layout>
    }
}
