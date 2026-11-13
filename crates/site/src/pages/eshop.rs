//! The e-shop: the product list with its filters, and the product pages
//! (firewood grid, lumber cards, simple variants).

use super::home::Bestseller;
use super::{loaded, PageMeta};
use crate::cart_islands::{AddTexts, AddToCart};
use crate::i18n::{t, Locale};
use crate::layout::{Crumbs, Layout};
use crate::variant_filter::{filter_texts, FilterGroup, FilterPanel};
use crate::visitor::{visitor, Visitor};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

/// Shop order when nothing has sold yet: the fallback for "Nejprodávanější".
#[cfg(feature = "ssr")]
const BESTSELLER_FALLBACK: [&str; 10] = ["buk", "dub", "smrk", "tramy", "fosny", "late", "piliny", "odkory", "stepka", "okrasne_kamenivo"];
const CATEGORIES: [&str; 4] = ["palivove", "rezivo", "zbytky", "kamenivo"];

/// A query parameter of the current address.
fn param(v: &Visitor, name: &str) -> Option<String> {
    v.query
        .as_deref()?
        .split('&')
        .find_map(|pair| pair.strip_prefix(name)?.strip_prefix('=').map(|value| value.replace('+', " ")))
        .filter(|value| !value.is_empty())
}

#[server(endpoint = "page/eshop")]
pub async fn load_eshop() -> Result<Vec<Bestseller>, ServerFnError> {
    let v = visitor();
    let db = &expect_context::<crate::state::AppState>().db;
    let category = param(&v, "category").filter(|c| CATEGORIES.contains(&c.as_str())).unwrap_or_else(|| "palivove".into());
    let hardness = match param(&v, "wood").as_deref() {
        Some("tvrde") => Some("hard"),
        Some("mekke") => Some("soft"),
        _ => None,
    };
    let fallback: &[&str] = if param(&v, "sort").as_deref() == Some("nejprodavanejsi") { &BESTSELLER_FALLBACK } else { &[] };
    let mut products = crate::catalog::bestsellers(db, &v, fallback).await.map_err(|e| ServerFnError::new(e.to_string()))?;
    products.retain(|p| p.category == category && hardness.is_none_or(|h| p.hardness.as_deref() == Some(h)));
    if fallback.is_empty() {
        // "Doporučujeme" is the order set in the admin.
        products.sort_by_key(|p| (p.position, p.id));
    }
    Ok(products)
}

/// A product card (the e-shop list and the recommendations).
#[component]
pub fn ProductCard(p: Bestseller, rank: Option<usize>) -> impl IntoView {
    let v = visitor();
    let l = v.locale;
    let href = v.link(&format!("/eshop/{}", p.key));
    view! {
        <div class="product-card">
            <a href=href.clone() class="product-image-wrapper-eshop">
                {rank.map(|r| view! { <span class="product-badge">{r}"."</span> })}
                {p.image.map(|src| view! { <img src=src alt=p.title.clone() class="product-img" loading="lazy"/> })}
            </a>
            <div class="product-info-eshop">
                <h4 class="product-title">{p.title}</h4>
                <p class="product-type">{p.type_label}</p>
                <p class="product-stock">{t::common::stock(l)}</p>
                <p class="product-price">{p.price_from.map_or_else(|| "---".to_owned(), |price| format!("{} {price}", t::common::price_from(l)))}</p>
                <a href=href class="btn-primary btn-detail">{t::common::detail(l)}</a>
            </div>
        </div>
    }
}

#[component]
pub fn EshopPage() -> impl IntoView {
    let v = visitor();
    let l = v.locale;
    let category_param = param(&v, "category");
    let category = category_param.clone().filter(|c| CATEGORIES.contains(&c.as_str())).unwrap_or_else(|| "palivove".into());
    let (title, description) = match category.as_str() {
        "rezivo" => (t::meta::eshop_rezivo::title(l), t::meta::eshop_rezivo::description(l)),
        "zbytky" => (t::meta::eshop_zbytky::title(l), t::meta::eshop_zbytky::description(l)),
        "kamenivo" => (t::meta::eshop_kamenivo::title(l), t::meta::eshop_kamenivo::description(l)),
        _ => (t::meta::eshop_palivove::title(l), t::meta::eshop_palivove::description(l)),
    };
    // The links keep the other choices, as the old ones did (empty when unset).
    let raw_category = category_param.unwrap_or_default();
    let wood = param(&v, "wood");
    let sort = param(&v, "sort");
    let (wood_q, sort_q) = (wood.clone().unwrap_or_default(), sort.clone().unwrap_or_default());
    let is = |current: &Option<String>, value: &str, default: bool| current.as_deref() == Some(value) || (default && current.is_none());
    let category_now = if raw_category.is_empty() { None } else { Some(raw_category.clone()) };
    use t::eshop::index as e;
    let link = |href: String, text: &'static str, class: &'static str, active: bool| {
        let class = if active { format!("{class} active") } else { class.to_owned() };
        view! { <a href=v.link(&href) class=class>{text}</a> }
    };
    let category_links = [
        ("palivove", e::cat_palivove(l), true),
        ("rezivo", e::cat_rezivo(l), false),
        ("zbytky", e::cat_zbytky(l), false),
        ("kamenivo", e::cat_kamenivo(l), false),
    ]
    .into_iter()
    .map(|(value, text, default)| link(format!("/eshop?category={value}&sort={sort_q}"), text, "btn-filter", is(&category_now, value, default)))
    .collect_view();
    let wood_links = (category == "palivove").then(|| {
        let links = [("vse", e::wood_all(l), true), ("tvrde", e::wood_hard(l), false), ("mekke", e::wood_soft(l), false)]
            .into_iter()
            .map(|(value, text, default)| link(format!("/eshop?category={raw_category}&wood={value}&sort={sort_q}"), text, "btn-filter btn-filter-sm", is(&wood, value, default)))
            .collect_view();
        view! {
            <h3 class="mt-40">{e::wood_type_title(l)}</h3>
            <div class="filter-group">{links}</div>
        }
    });
    let sort_links = [("doporucujeme", e::sort_recommended(l), true), ("nejprodavanejsi", e::sort_bestsellers(l), false)]
        .into_iter()
        .map(|(value, text, default)| link(format!("/eshop?category={raw_category}&wood={wood_q}&sort={value}"), text, "btn-sort", is(&sort, value, default)))
        .collect_view();
    view! {
        <PageMeta title=title.to_owned() description=description keep_query=vec!["category"]/>
        <Layout>
            <main class="eshop-main main">
                <div class="eshop-container">
                    <section class="eshop-filters">
                        <h3>{e::filters_title(l)}</h3>
                        <div class="filter-group">{category_links}</div>
                        {wood_links}
                    </section>
                    <section class="eshop-sorting">
                        <div class="sorting-tabs">{sort_links}</div>
                        <hr class="sorting-divider"/>
                    </section>
                    <DeliveryNote/>
                    {loaded(load_eshop(), |products: Vec<Bestseller>| view! {
                        <section class="eshop-grid">
                            {products.into_iter().enumerate().map(|(i, p)| view! { <ProductCard p=p rank=(i < 3).then_some(i + 1)/> }).collect_view()}
                        </section>
                    })}
                </div>
                <CustomDemand/>
            </main>
        </Layout>
    }
}

#[component]
pub fn CustomDemand() -> impl IntoView {
    let v = visitor();
    let l = v.locale;
    use t::shared::custom_demand as c;
    view! {
        <section class="custom-demand-section">
            <h2 class="custom-demand-title">{c::title_line1(l)}" "<span class="text-blue">{c::title_line2(l)}</span></h2>
            <div class="custom-demand-card">
                <div class="custom-demand-img-wrapper">
                    <img src="/img/eshop/custom-production-banner.webp" alt=c::banner_alt(l) class="custom-demand-banner-img" loading="lazy"/>
                </div>
                <div class="custom-demand-content">
                    <div class="custom-demand-text">
                        <h3>{c::card_title(l)}</h3>
                        <p inner_html=c::text(l).into_string()></p>
                    </div>
                    <div class="custom-demand-action">
                        <a href=v.link("/kontakt#kalkulace") class="icon-only-btn">
                            <svg viewBox="0 0 24 24" width="30" height="30" fill="white"><path d="M8.59 16.59L13.17 12 8.59 7.41 10 6l6 6-6 6-1.41-1.41z"/></svg>
                        </a>
                    </div>
                </div>
            </div>
        </section>
    }
}

/// "Kam vozíme?": prices are for the goods only; a town or PSČ gives the
/// approximate delivery price at once (checkout recalculates it with the
/// full address).
#[component]
pub fn DeliveryNote() -> impl IntoView {
    let v = visitor();
    let l = v.locale;
    let rate = v.shop.price(l, domain::Czk(v.shop.czk_per_km));
    use t::delivery_estimate as d;
    let body = match v.delivery.clone() {
        Some(estimate) => {
            let quote = estimate.quote(v.shop.czk_per_km);
            view! {
                <p class="delivery-estimate-result" inner_html=d::result(l, quote.one_way_km, &estimate.place, &v.shop.price(l, quote.price()), &rate, quote.round_trip_km()).into_string()></p>
                <p class="delivery-estimate-hint">{d::result_hint(l)}</p>
                <form class="button_to" method="post" action={v.link("/doprava-odhad/zrusit")}>
                    <button class="delivery-estimate-change" type="submit">{d::change(l)}</button>
                </form>
            }
            .into_any()
        }
        None => {
            let country = v.user.as_ref().and_then(|u| u.billing_country.clone()).filter(|c| !c.is_empty()).unwrap_or_else(|| if l == Locale::De { "at".into() } else { "cz".into() });
            view! {
                <p inner_html=t::shared::delivery_note(l, &rate).into_string()></p>
                <form action={v.link("/doprava-odhad")} method="post" class="delivery-estimate-form">
                    <label class="visually-hidden" for="delivery-estimate-query">{d::query_label(l)}</label>
                    <div class="delivery-estimate-place address-autocomplete">
                        <crate::autocomplete::PlaceInput
                            id="delivery-estimate-query".to_owned()
                            value=v.user.as_ref().and_then(|u| u.billing_city.clone()).unwrap_or_default()
                            placeholder=d::query_placeholder(l).to_owned()
                            country_select="delivery-estimate-country".to_owned()
                            suggestions_url="/naseptavac-adres".to_owned()
                        />
                    </div>
                    <label class="visually-hidden" for="delivery-estimate-country">{t::checkout::country_field(l)}</label>
                    <select id="delivery-estimate-country" name="country">
                        <option value="cz" selected=country == "cz">{d::country_cz(l)}</option>
                        <option value="at" selected=country == "at">{d::country_at(l)}</option>
                    </select>
                    <button type="submit" class="btn-primary">{d::submit(l)}</button>
                </form>
                {v.delivery_error.clone().map(|error| view! { <p class="delivery-estimate-error" role="alert">{error}</p> })}
            }
            .into_any()
        }
    };
    view! {
        <div class="delivery-note" id="delivery-estimate">
            <svg class="delivery-note-icon" viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="1" y="3" width="15" height="13"></rect><polygon points="16 8 20 8 23 11 23 16 16 16 16 8"></polygon><circle cx="5.5" cy="18.5" r="2.5"></circle><circle cx="18.5" cy="18.5" r="2.5"></circle></svg>
            <div class="delivery-note-body">{body}</div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Product pages

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Variant {
    id: i64,
    price: String,
    in_stock: bool,
    stacked: bool,
    group: Option<String>,
    length: Option<String>,
    amount: Option<i64>,
    grade: Option<String>,
    grade_key: Option<String>,
    width: Option<i64>,
    height: Option<i64>,
    label: String,
    image: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Product {
    key: String,
    template: String,
    category: String,
    title: String,
    type_label: String,
    subtitle: String,
    description: String,
    drying_note: String,
    image: Option<String>,
    image_alt: String,
    variants: Vec<Variant>,
    recommendations: Vec<Bestseller>,
}

#[server(endpoint = "page/product")]
pub async fn load_product(key: String) -> Result<Option<Product>, ServerFnError> {
    use crate::shop::translated as tr;
    use domain::money::Czk;
    let v = visitor();
    let l = v.locale;
    let db = &expect_context::<crate::state::AppState>().db;
    let err = |e: sqlx::Error| ServerFnError::new(e.to_string());
    let Some(p) = sqlx::query!(r#"SELECT id AS "id!", key, template, category, title, title_de, type_label, type_label_de, subtitle, subtitle_de,
                                         description, description_de, drying_note, drying_note_de, image, image_alt, image_alt_de
                                  FROM catalog_products WHERE key = ? AND active = 1"#, key)
        .fetch_optional(db)
        .await
        .map_err(err)?
    else {
        return Ok(None);
    };
    let category: domain::Category = p.category.parse().map_err(|_| ServerFnError::new("bad category"))?;
    let text = |cs: Option<&str>, de: Option<&str>| tr(l, cs.unwrap_or_default(), de).to_owned();
    let variants = sqlx::query!(
        r#"SELECT id AS "id!", price_czk, in_stock, stacked, variant_group, length_label, amount_value, grade, grade_de,
                  width_mm, height_mm, variant_label, variant_label_de, image
           FROM catalog_variants WHERE catalog_product_id = ? ORDER BY position, id"#,
        p.id
    )
    .fetch_all(db)
    .await
    .map_err(err)?
    .into_iter()
    .map(|r| Variant {
        id: r.id,
        price: v.shop.price(l, v.shop.vat.customer_price(category, Czk(r.price_czk))),
        in_stock: r.in_stock != 0,
        stacked: r.stacked != 0 || r.variant_group.as_deref() == Some("bedny"),
        group: r.variant_group,
        length: r.length_label,
        amount: r.amount_value,
        grade_key: r.grade.as_deref().filter(|g| !g.is_empty()).map(super::info::parameterize),
        grade: r.grade.as_deref().filter(|g| !g.is_empty()).map(|g| tr(l, g, r.grade_de.as_deref()).to_owned()),
        width: r.width_mm,
        height: r.height_mm,
        label: text(r.variant_label.as_deref(), r.variant_label_de.as_deref()),
        image: r.image,
    })
    .collect();
    // Four others, picked at random each time.
    let mut others = crate::catalog::bestsellers(db, &v, &[]).await.map_err(err)?;
    others.retain(|o| o.key != p.key);
    {
        use rand::seq::SliceRandom;
        others.shuffle(&mut rand::rng());
    }
    others.truncate(4);
    Ok(Some(Product {
        key: p.key,
        template: p.template,
        category: p.category,
        title: text(Some(&p.title), p.title_de.as_deref()),
        type_label: text(p.type_label.as_deref(), p.type_label_de.as_deref()),
        subtitle: text(p.subtitle.as_deref(), p.subtitle_de.as_deref()),
        description: text(p.description.as_deref(), p.description_de.as_deref()),
        drying_note: text(p.drying_note.as_deref(), p.drying_note_de.as_deref()),
        image: p.image,
        image_alt: text(p.image_alt.as_deref(), p.image_alt_de.as_deref()),
        variants,
        recommendations: others,
    }))
}

fn add_texts(v: &Visitor) -> AddTexts {
    let l = v.locale;
    AddTexts {
        action: v.link("/cart_items"),
        add: t::common::add_to_cart(l).into(),
        decrease: t::common::qty_decrease(l).into(),
        increase: t::common::qty_increase(l).into(),
        added_title: t::cart::added_title(l).into(),
        close: t::common::close(l).into(),
        continue_shopping: t::cart::continue_shopping(l).into(),
        to_checkout: t::cart::to_checkout(l).into(),
        cart_href: v.link("/kosik"),
    }
}

/// The add-to-cart form, or the grey "out of stock" button.
fn buy(v: &Visitor, variant: &Variant, show_qty: bool) -> AnyView {
    if variant.in_stock {
        view! { <AddToCart variant_id=variant.id show_qty=show_qty texts=add_texts(v)/> }.into_any()
    } else {
        view! {
            <form class="button_to" method="post" action="#">
                <button class="btn-primary btn-cart" disabled style="background-color: #9CA3AF; color: #FFFFFF; border: none; cursor: not-allowed;" type="submit">
                    {t::common::out_of_stock(v.locale)}
                </button>
            </form>
        }
        .into_any()
    }
}

#[component]
pub fn ProductPage() -> impl IntoView {
    let params = use_params_map();
    let key = params.with_untracked(|p| p.get("key").unwrap_or_default());
    view! {
        {loaded(load_product(key), |product: Option<Product>| match product {
            Some(product) => view! { <ProductView product=product/> }.into_any(),
            None => view! { <super::not_found::NotFoundPage/> }.into_any(),
        })}
    }
}

#[component]
fn ProductView(product: Product) -> impl IntoView {
    let v = visitor();
    let l = v.locale;
    let title = product.title.clone();
    let meta_title = [product.title.as_str(), product.type_label.as_str()].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" – ");
    let description = {
        let plain = strip_tags(&product.description);
        if plain.trim().is_empty() { t::meta::product_fallback(l, &product.title) } else { plain }
    };
    // E-shop › category › product (the category is left out when it's the product's own name).
    let category_name = match product.category.as_str() {
        "palivove" => t::breadcrumbs::palivove_drevo(l),
        "rezivo" => t::breadcrumbs::stavebni_rezivo(l),
        "zbytky" => t::breadcrumbs::vyrobni_zbytky(l),
        _ => t::breadcrumbs::okrasne_kamenivo(l),
    };
    let mut crumbs = vec![(t::breadcrumbs::eshop(l).to_owned(), "/eshop".to_owned())];
    if category_name != title {
        crumbs.push((category_name.to_owned(), format!("/eshop?category={}", product.category)));
    }
    crumbs.push((title.clone(), "#".to_owned()));
    provide_context(Crumbs(crumbs));

    let intro = {
        let alt = if product.template == "firewood" { product.image_alt.clone() } else { title.clone() };
        let firewood = product.template == "firewood";
        let stock = match product.template.as_str() {
            "firewood" => Some(t::eshop::firewood::stock_status(l)),
            "lumber" => Some(t::common::in_stock(l)),
            _ => None,
        };
        view! {
            <section class="product-intro-section">
                <div class="product-intro-image">
                    {product.image.clone().map(|src| view! { <img src=src alt=alt loading="eager" fetchpriority="high"/> })}
                </div>
                <div class="product-intro-details">
                    <h1>{title.clone()}</h1>
                    {firewood.then(|| view! { <p class="subtitle">{product.subtitle.clone()}</p> })}
                    <p class="description">{product.description.clone()}</p>
                    {firewood.then(|| view! { <p class="drying-note">{product.drying_note.clone()}</p> })}
                    {stock.map(|s| view! { <p class="stock-status">{s}</p> })}
                    <DeliveryNote/>
                </div>
            </section>
        }
    };
    let inventory = match product.template.as_str() {
        "firewood" => firewood_inventory(&v, &product).into_any(),
        "lumber" => lumber_inventory(&v, &product).into_any(),
        _ => simple_inventory(&v, &product).into_any(),
    };
    view! {
        <PageMeta title=meta_title description=description/>
        <Layout>
            <main class="product-page-main main">
                <div class="product-container">
                    {intro}
                    {inventory}
                    <section class="recommendations-section">
                        <h2 class="section-title text-center" inner_html=t::shared::recommendations::title(l).into_string()></h2>
                        <div class="eshop-grid">
                            {product.recommendations.into_iter().map(|p| view! { <ProductCard p=p rank=None/> }).collect_view()}
                        </div>
                    </section>
                </div>
                <CustomDemand/>
            </main>
        </Layout>
    }
}

/// Text without its tags (for a meta description).
fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

fn spec_rows(labels: Vec<&'static str>, values: Vec<String>) -> impl IntoView {
    view! {
        <div class="inventory-specs">
            <div class="spec-labels">{labels.into_iter().map(|s| view! { <span>{s}</span> }).collect_view()}</div>
            <div class="spec-values">{values.into_iter().map(|s| view! { <span class="spec-value">{s}</span> }).collect_view()}</div>
        </div>
    }
}

/// The leading number of "33 cm" (for sorting and the filter): 33.
fn leading_number(label: &str) -> i64 {
    label.chars().filter(|c| !c.is_whitespace()).take_while(char::is_ascii_digit).collect::<String>().parse().unwrap_or(0)
}

/// Firewood: loose/stacked in containers as a grid (rows of amounts ×
/// columns of lengths), then crates.
fn firewood_inventory(v: &Visitor, product: &Product) -> impl IntoView {
    let l = v.locale;
    let containers: Vec<&Variant> = product.variants.iter().filter(|x| x.group.as_deref() == Some("kontejner")).collect();
    let crates: Vec<&Variant> = product.variants.iter().filter(|x| x.group.as_deref() == Some("bedny")).collect();

    // The grid: lengths in order of first appearance; each row holds the
    // n-th smallest amount of each kind (loose / stacked).
    let mut lengths: Vec<String> = Vec::new();
    for x in &containers {
        let len = x.length.clone().unwrap_or_default();
        if !lengths.contains(&len) {
            lengths.push(len);
        }
    }
    let mut kinds: Vec<(bool, Vec<i64>)> = Vec::new();
    for x in &containers {
        let entry = match kinds.iter_mut().find(|(k, _)| *k == x.stacked) {
            Some(e) => e,
            None => {
                kinds.push((x.stacked, Vec::new()));
                kinds.last_mut().unwrap_or_else(|| unreachable!())
            }
        };
        if let Some(a) = x.amount {
            if !entry.1.contains(&a) {
                entry.1.push(a);
            }
        }
    }
    for (_, amounts) in &mut kinds {
        amounts.sort_unstable();
    }
    let row_count = kinds.iter().map(|(_, a)| a.len()).max().unwrap_or(0);
    let variant_at = |row: usize, length: &str| {
        containers.iter().copied().find(|x| {
            x.length.as_deref().unwrap_or_default() == length
                && kinds.iter().find(|(k, _)| *k == x.stacked).and_then(|(_, a)| a.get(row)).copied() == x.amount
        })
    };

    let all: Vec<&Variant> = containers.iter().chain(crates.iter()).copied().collect();
    let mut filter_lengths: Vec<String> = Vec::new();
    for x in &all {
        let len = x.length.clone().unwrap_or_default();
        if !filter_lengths.contains(&len) {
            filter_lengths.push(len);
        }
    }
    filter_lengths.sort_by_key(|len| -leading_number(len));
    let mut amounts: Vec<i64> = all.iter().filter_map(|x| x.amount).collect();
    amounts.sort_unstable();
    amounts.dedup();
    let kinds_present: Vec<&str> = ["loose", "stacked"].into_iter().filter(|k| all.iter().any(|x| x.stacked == (*k == "stacked"))).collect();
    let groups: Vec<FilterGroup> = vec![
        (
            "kind".into(),
            t::common::spec_variant(l).into(),
            kinds_present
                .into_iter()
                .map(|k| (k.to_owned(), None, if k == "stacked" { t::common::stacked_variant(l) } else { t::common::loose_variant(l) }.to_owned()))
                .collect(),
        ),
        ("amount".into(), t::common::spec_amount(l).into(), amounts.iter().map(|a| (a.to_string(), Some(a.to_string()), t::common::per_m3(l).to_owned())).collect()),
        (
            "length".into(),
            t::common::spec_length(l).into(),
            filter_lengths
                .iter()
                .map(|len| (len.clone(), Some(leading_number(len).to_string()), len.trim_start_matches(|c: char| c.is_ascii_digit() || c.is_whitespace()).to_owned()))
                .collect(),
        ),
    ];

    let card = |x: &Variant, length: &str, stacked: bool, fallback: &'static str, alt: &'static str| {
        let kind = if stacked { "stacked" } else { "loose" };
        let amount = x.amount.map(|a| a.to_string()).unwrap_or_default();
        view! {
            <div class="inventory-card" data-kind=kind data-amount=amount.clone() data-length=length.to_owned()>
                <img src=x.image.clone().unwrap_or_else(|| format!("/img/{fallback}")) alt=alt class="inventory-img" loading="lazy"/>
                {spec_rows(
                    vec![t::common::spec_variant(l), t::common::spec_length(l), t::common::spec_amount(l)],
                    vec![
                        if stacked { t::common::stacked_variant(l) } else { t::common::loose_variant(l) }.to_owned(),
                        length.to_owned(),
                        format!("{amount} {}", t::common::per_m3(l)),
                    ],
                )}
                <p class="inventory-price">{x.price.clone()}</p>
                {buy(v, x, false)}
            </div>
        }
    };

    view! {
        <section class="inventory-section">
            <FilterPanel groups=groups texts=filter_texts(l)/>
            <hr class="inventory-separator main-separator"/>
            <div id="section-kontejner" class="inventory-group" data-filter-section="">
                <h2 class="section-title" inner_html=t::eshop::firewood::variants_heading(l).into_string()></h2>
                <div class="matrix-grid" data-filter-grid="" data-columns="4">
                    {(0..row_count).flat_map(|row| lengths.iter().map(move |len| (row, len.clone()))).map(|(row, len)| view! {
                        <div class="matrix-cell" data-row=row.to_string() data-length=len.clone()>
                            {variant_at(row, &len).map(|x| card(x, &len, x.stacked, "eshop/container.webp", t::common::container(l)))}
                        </div>
                    }).collect_view()}
                </div>
            </div>
            <hr class="inventory-separator mid-separator" data-filter-separator=""/>
            <div id="section-bedny" class="inventory-group" data-filter-section="">
                <h2 class="section-title" inner_html=t::eshop::firewood::crates_heading(l).into_string()></h2>
                <div class="matrix-grid-bedny" data-filter-grid="" data-columns="3">
                    {crates.iter().map(|x| {
                        let len = x.length.clone().unwrap_or_default();
                        view! {
                            <div class="matrix-cell" data-row="crates" data-length=len.clone()>
                                {card(x, &len, true, "eshop/bedna.webp", t::common::crate_(l))}
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}

/// Lumber: one card per size with grade, length, width and height.
fn lumber_inventory(v: &Visitor, product: &Product) -> impl IntoView {
    let l = v.locale;
    let sizes = |pick: fn(&Variant) -> Option<i64>| {
        let mut mm: Vec<i64> = product.variants.iter().filter_map(pick).collect();
        mm.sort_unstable();
        mm.dedup();
        mm.into_iter().map(|m| (m.to_string(), Some(m.to_string()), "mm".to_owned())).collect::<Vec<_>>()
    };
    let mut lengths: Vec<String> = Vec::new();
    for x in &product.variants {
        if let Some(len) = &x.length {
            if !lengths.contains(len) {
                lengths.push(len.clone());
            }
        }
    }
    lengths.sort_by_key(|len| leading_number(len));
    let length_options = lengths
        .iter()
        .map(|len| {
            let digits: String = len.chars().take_while(|c| c.is_ascii_digit() || c.is_whitespace()).collect();
            (len.clone(), Some(digits.trim().to_owned()).filter(|d| !d.is_empty()), len[digits.len()..].to_owned())
        })
        .collect::<Vec<_>>();
    let mut grades: Vec<(String, Option<String>, String)> = Vec::new();
    for x in &product.variants {
        if let (Some(key), Some(grade)) = (&x.grade_key, &x.grade) {
            if !grades.iter().any(|(k, _, _)| k == key) {
                grades.push((key.clone(), None, grade.clone()));
            }
        }
    }
    let groups: Vec<FilterGroup> = vec![
        ("width".into(), t::common::spec_width(l).into(), sizes(|x| x.width)),
        ("height".into(), t::common::spec_height(l).into(), sizes(|x| x.height)),
        ("length".into(), t::common::spec_length(l).into(), length_options),
        ("grade".into(), t::common::spec_grade(l).into(), grades),
    ]
    .into_iter()
    .filter(|(_, _, options)| options.len() > 1)
    .collect();
    let has_filter = !groups.is_empty();
    let title = product.title.clone();
    let cards = product
        .variants
        .iter()
        .map(|x| {
            let mut labels = Vec::new();
            let mut values = Vec::new();
            if let Some(grade) = &x.grade {
                labels.push(t::common::spec_grade(l));
                values.push(grade.clone());
            }
            labels.push(t::common::spec_length(l));
            values.push(x.length.clone().unwrap_or_default());
            if let Some(w) = x.width {
                labels.push(t::common::spec_width(l));
                values.push(format!("{w} mm"));
            }
            if let Some(h) = x.height {
                labels.push(t::common::spec_height(l));
                values.push(format!("{h} mm"));
            }
            view! {
                <div class="inventory-card"
                    data-width=x.width.map(|w| w.to_string()).unwrap_or_default()
                    data-height=x.height.map(|h| h.to_string()).unwrap_or_default()
                    data-length=x.length.clone().unwrap_or_default()
                    data-grade=x.grade_key.clone().unwrap_or_default()>
                    {x.image.clone().or_else(|| product.image.clone()).map(|src| view! { <img src=src alt=title.clone() class="inventory-img" loading="lazy"/> })}
                    {spec_rows(labels, values)}
                    <p class="inventory-price">{x.price.clone()}" "{t::common::price_per_piece(l)}</p>
                    {buy(v, x, true)}
                </div>
            }
        })
        .collect_view();
    let heading = view! { <h2 class="section-title" inner_html=t::eshop::lumber::variants_heading(l).into_string()></h2> };
    view! {
        <section class="inventory-section">
            <hr class="inventory-separator main-separator"/>
            {heading}
            {has_filter.then(|| view! { <FilterPanel groups=groups texts=filter_texts(l)/> })}
            <div class="simple-grid-3" id="lumberGrid" data-filter-grid="">{cards}</div>
        </section>
    }
}

/// Simple products (by-products, stone): a card per variant.
fn simple_inventory(v: &Visitor, product: &Product) -> impl IntoView {
    let l = v.locale;
    let grid = if product.variants.len() <= 2 { "simple-grid-2" } else { "simple-grid-3" };
    view! {
        <section class="inventory-section">
            <hr class="inventory-separator main-separator"/>
            <div id="section-kontejner" class="inventory-group">
                <h2 class="section-title" inner_html=t::common::available_variants(l).into_string()></h2>
                <div class=grid>
                    {product.variants.iter().map(|x| view! {
                        <div class="inventory-card">
                            <img src=x.image.clone().unwrap_or_else(|| "/img/eshop/container.webp".into()) alt=t::common::container(l) class="inventory-img" loading="lazy"/>
                            {spec_rows(vec![t::common::spec_description(l), t::common::spec_variant(l)], vec![product.type_label.clone(), x.label.clone()])}
                            <p class="inventory-price">{x.price.clone()}</p>
                            <p class=if x.in_stock { "spec-value text-green" } else { "spec-value text-red" } style="margin-bottom: 12px; font-size: 0.95rem;">
                                {if x.in_stock { t::common::in_stock(l) } else { t::common::out_of_stock(l) }}
                            </p>
                            {buy(v, x, true)}
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}
