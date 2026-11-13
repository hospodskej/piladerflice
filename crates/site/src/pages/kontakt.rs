//! The contact page: contact details, the inquiry form and the price list.

use super::{loaded, PageMeta, SafeHtml};
use crate::i18n::t;
use crate::inquiry_form::{InquiryForm, InquiryFormData};
#[cfg(feature = "ssr")]
use crate::inquiry_form::{FormCategory, FormField, FormTexts};
use crate::layout::Layout;
use crate::visitor::visitor;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// The fields of each inquiry category, in order (dropdowns, plus the
/// lumber quantity box).
pub const ROW_FIELDS: [(&str, &[&str]); 2] = [
    ("palivove", &["varianta", "druh", "delka", "mnozstvi"]),
    ("stavebni", &["polozka", "delka", "vyska", "sirka", "mnozstvi"]),
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PriceRow {
    name: SafeHtml,
    details: SafeHtml,
    price: SafeHtml,
    /// The price isn't "---" (firewood rows get "/ m³" after it).
    priced: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KontaktContent {
    stavebni: Vec<PriceRow>,
    palivove_volne: Vec<(SafeHtml, Vec<PriceRow>)>,
    palivove_skladane: Vec<(SafeHtml, Vec<PriceRow>)>,
    zbytky: Vec<PriceRow>,
    kamenivo: Option<SafeHtml>,
    sluzby: Vec<PriceRow>,
    form: InquiryFormData,
}

#[server(endpoint = "page/kontakt")]
pub async fn load_kontakt() -> Result<KontaktContent, ServerFnError> {
    use crate::i18n::Locale;
    use crate::shop::translated as tr;
    let v = visitor();
    let l = v.locale;
    let db = &expect_context::<crate::state::AppState>().db;
    let err = |e: sqlx::Error| ServerFnError::new(e.to_string());

    let items = sqlx::query!(
        "SELECT category, delivery_rate, subcategory, subcategory_de, item_name, item_name_de, details, details_de, price, price_de
         FROM pricelist_items ORDER BY position, id"
    )
    .fetch_all(db)
    .await
    .map_err(err)?;

    let mut content = KontaktContent {
        stavebni: vec![],
        palivove_volne: vec![],
        palivove_skladane: vec![],
        zbytky: vec![],
        kamenivo: None,
        sluzby: vec![],
        form: inquiry_form(db, &v).await.map_err(err)?,
    };
    for item in items {
        let firewood = item.category.starts_with("palivove");
        let price = if item.delivery_rate != 0 {
            v.shop.price_text(l, &format!("{} Kč / km", v.shop.czk_per_km))
        } else if l == Locale::De && firewood {
            // German firewood prices are typed in by hand, in euros.
            tr(l, &item.price, item.price_de.as_deref()).to_owned()
        } else {
            v.shop.price_text(l, tr(l, &item.price, item.price_de.as_deref()))
        };
        let row = PriceRow {
            name: SafeHtml::from_admin(tr(l, item.item_name.as_deref().unwrap_or_default(), item.item_name_de.as_deref())),
            details: SafeHtml::from_admin(tr(l, item.details.as_deref().unwrap_or_default(), item.details_de.as_deref())),
            priced: price != "---",
            price: SafeHtml::from_admin(&price),
        };
        let subcategory = SafeHtml::from_admin(tr(l, item.subcategory.as_deref().unwrap_or_default(), item.subcategory_de.as_deref()));
        let group = |groups: &mut Vec<(SafeHtml, Vec<PriceRow>)>, row: PriceRow| match groups.iter_mut().find(|(s, _)| *s == subcategory) {
            Some((_, rows)) => rows.push(row),
            None => groups.push((subcategory.clone(), vec![row])),
        };
        match item.category.as_str() {
            "stavebni" => content.stavebni.push(row),
            "palivove_volne" => group(&mut content.palivove_volne, row),
            "palivove_skladane" => group(&mut content.palivove_skladane, row),
            "zbytky" => content.zbytky.push(row),
            "kamenivo" if content.kamenivo.is_none() => content.kamenivo = Some(row.price),
            "sluzby" => content.sluzby.push(row),
            _ => {}
        }
    }
    Ok(content)
}

#[cfg(feature = "ssr")]
async fn inquiry_form(db: &sqlx::SqlitePool, v: &crate::visitor::Visitor) -> Result<InquiryFormData, sqlx::Error> {
    use crate::shop::translated as tr;
    let l = v.locale;
    let options = sqlx::query!("SELECT category, field, value, value_de FROM inquiry_form_options ORDER BY position, id")
        .fetch_all(db)
        .await?;
    let label = |field: &str| match field {
        "varianta" => t::kontakt::grid_varianta(l),
        "druh" => t::kontakt::grid_druh(l),
        "delka" => t::kontakt::grid_delka(l),
        "mnozstvi" => t::kontakt::grid_mnozstvi(l),
        "polozka" => t::kontakt::grid_polozka(l),
        "vyska" => t::kontakt::grid_vyska(l),
        _ => t::kontakt::grid_sirka(l),
    };
    let categories = ROW_FIELDS
        .iter()
        .map(|(category, fields)| FormCategory {
            key: (*category).to_owned(),
            title: if *category == "palivove" { t::kontakt::toggle_palivove(l) } else { t::kontakt::toggle_stavebni(l) }.to_owned(),
            fields: fields
                .iter()
                .map(|field| FormField {
                    key: (*field).to_owned(),
                    label: label(field).to_owned(),
                    options: options
                        .iter()
                        .filter(|o| o.category == *category && o.field == *field)
                        .map(|o| tr(l, &o.value, o.value_de.as_deref()).to_owned())
                        .collect(),
                    custom: *category == "stavebni" || *field == "mnozstvi",
                    number: *category == "stavebni" && *field == "mnozstvi",
                })
                .collect(),
        })
        .collect();
    let privacy_link = codon::Html::from_trusted(format!(
        r#"<a href="{}" target="_blank" rel="noopener">{}</a>"#,
        v.link("/zasady-ochrany-osobnich-udaju"),
        codon::Html::escape(t::kontakt::privacy_link_text(l))
    ));
    use t::kontakt as k;
    Ok(InquiryFormData {
        categories,
        texts: FormTexts {
            action: v.link("/inquiries"),
            toggle_hint: k::toggle_hint(l).to_owned(),
            section_formular: k::form_section_formular(l).to_owned(),
            section_poznamky: k::form_section_poznamky(l).to_owned(),
            custom_option: k::kalkulace_custom_option(l).to_owned(),
            custom_placeholder: k::kalkulace_custom_placeholder(l).to_owned(),
            quantity_placeholder: k::kalkulace_mnozstvi_placeholder(l).to_owned(),
            delete_item: k::delete_item_aria(l).to_owned(),
            add_item: k::add_item_aria(l).to_owned(),
            consent_html: k::consent_text(l, &privacy_link).into_string(),
            submit: k::submit_button(l).to_owned(),
        },
    })
}

fn price_table(rows: Vec<PriceRow>, columns: usize, per_m3: Option<&'static str>, left_name: bool) -> impl IntoView {
    view! {
        <table class=if columns == 2 { "cenik-table two-cols" } else { "cenik-table" }>
            <tbody>
                {rows.into_iter().map(|row| {
                    let suffix = per_m3.filter(|_| row.priced).map(|unit| format!(" / {unit}"));
                    view! {
                        <tr>
                            <td style=left_name.then_some("text-align: left") inner_html=row.name.into_string()></td>
                            {(columns == 3).then(|| view! { <td inner_html=row.details.into_string()></td> })}
                            <td><span inner_html=row.price.into_string()></span>{suffix}</td>
                        </tr>
                    }
                }).collect_view()}
            </tbody>
        </table>
    }
}

fn price_group(title: &'static str, body: impl IntoView) -> impl IntoView {
    view! {
        <div class="cenik-group">
            <div class="cenik-header"><h3>{title}</h3><span class="cenik-line"></span></div>
            {body}
        </div>
    }
}

fn firewood_groups(groups: Vec<(SafeHtml, Vec<PriceRow>)>, per_m3: &'static str) -> impl IntoView {
    groups
        .into_iter()
        .map(|(name, rows)| view! {
            <div class="cenik-subgroup">
                <h4 class="cenik-subtitle" inner_html=name.into_string()></h4>
                {price_table(rows, 2, Some(per_m3), false)}
            </div>
        })
        .collect_view()
}

#[component]
pub fn KontaktPage() -> impl IntoView {
    let l = visitor().locale;
    use t::kontakt as k;
    let info = |icon: &'static str, text: AnyView| view! {
        <li>
            <svg viewBox="0 0 24 24" fill="currentColor" class="kontakt-icon" inner_html=icon></svg>
            {text}
        </li>
    };
    view! {
        <PageMeta title=t::meta::kontakt::title(l).to_owned() description=t::meta::kontakt::description(l)/>
        <crate::seo::LocalBusinessSchema/>
        <Layout>
            <main class="main">
                <section class="kontakt-page-section">
                    <h1 class="kontakt-main-title">{k::page_title(l)}</h1>
                    <div class="kontakt-layout-grid">
                        <div class="kontakt-image-wrapper">
                            <img src="/img/kontakt/drevo.webp" alt=k::image_alt(l) class="kontakt-image" loading="eager" fetchpriority="high"/>
                        </div>
                        <div class="kontakt-details-wrapper">
                            <div class="kontakt-titles">
                                <h2>{k::brand_name(l)}</h2>
                                <h3 style="color: #2C8FFF">{k::brand_sub(l)}</h3>
                            </div>
                            <div class="kontakt-info-group">
                                <h4>{k::kontakt_title(l)}</h4>
                                <ul class="kontakt-list-items">
                                    {info(r#"<path d="M17 2H7c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h10c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm-5 18c-.83 0-1.5-.67-1.5-1.5s.67-1.5 1.5-1.5 1.5.67 1.5 1.5-.67 1.5-1.5 1.5z"/>"#, view! { <a href="tel:+420602446339">"+420 602 446 339"</a> }.into_any())}
                                    {info(r#"<path d="M20 15.5c-1.25 0-2.45-.2-3.57-.57a1.02 1.02 0 0 0-1.02.24l-2.2 2.2a15.045 15.045 0 0 1-6.59-6.59l2.2-2.21a.96.96 0 0 0 .25-1A11.36 11.36 0 0 1 8.5 4c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1 0 9.39 7.61 17 17 17 .55 0 1-.45 1-1v-3.5c0-.55-.45-1-1-1z"/>"#, view! { <a href="tel:+420515235527">"+420 515 235 527"</a> }.into_any())}
                                    {info(r#"<path d="M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>"#, view! { <a href="mailto:stepan.merta@seznam.cz">"stepan.merta@seznam.cz"</a> }.into_any())}
                                    {info(r#"<path d="M12 2C8.13 2 5 5.13 5 9c0 5.25 7 13 7 13s7-7.75 7-13c0-3.87-3.13-7-7-7zm0 9.5c-1.38 0-2.5-1.12-2.5-2.5s1.12-2.5 2.5-2.5 2.5 1.12 2.5 2.5-1.12 2.5-2.5 2.5z"/>"#, view! { <span>{k::address(l)}</span> }.into_any())}
                                </ul>
                            </div>
                            <div class="kontakt-info-group">
                                <h4>{k::hours_title(l)}</h4>
                                <ul class="kontakt-list-items">
                                    {info(r#"<path d="M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67z"/>"#, view! { <span inner_html=k::hours_text(l).into_string()></span> }.into_any())}
                                </ul>
                            </div>
                            <div class="kontakt-info-group">
                                <h4>{k::info_title(l)}</h4>
                                <ul class="kontakt-list-items">
                                    {info(r#"<path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/>"#, view! { <span>{k::info_responsible(l)}</span> }.into_any())}
                                    {info(r#"<path d="M20 6h-4V4c0-1.11-.89-2-2-2h-4c-1.11 0-2 .89-2 2v2H4c-1.11 0-1.99.89-1.99 2L2 19c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2zm-6 0h-4V4h4v2z"/>"#, view! { <span>{k::info_ico(l)}</span> }.into_any())}
                                    {info(r#"<path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/>"#, view! { <span>{k::info_billing(l)}</span> }.into_any())}
                                    {info(r#"<path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>"#, view! { <span>{k::info_legal(l)}</span> }.into_any())}
                                </ul>
                            </div>
                        </div>
                    </div>
                </section>
                {loaded(load_kontakt(), move |c: KontaktContent| {
                    let per_m3 = t::common::per_m3(l);
                    let field = |name: &'static str, label: &'static str, kind: &'static str, autocomplete: Option<&'static str>| view! {
                        <div class="form-group">
                            <label for=name>{label}</label>
                            <input type=kind name=name id=name required autocomplete=autocomplete/>
                        </div>
                    };
                    view! {
                        <section class="kalkulace" id="kalkulace">
                            <div class="kalkulace-header">
                                <h2 inner_html=k::kalkulace_title(l).into_string()></h2>
                            </div>
                            <InquiryForm data=c.form>
                                <h3 class="form-section-title">{k::form_section_kontakt(l)}</h3>
                                // Row by row, as they are read and filled in: name, how
                                // to reach you, then the address.
                                <div class="kalkulace-grid-2">
                                    {field("jmeno", k::form_jmeno(l), "text", Some("given-name"))}
                                    {field("prijmeni", k::form_prijmeni(l), "text", Some("family-name"))}
                                    {field("email", k::form_email(l), "email", Some("email"))}
                                    {field("telefon", k::form_telefon(l), "tel", Some("tel"))}
                                    {field("ulice", k::form_ulice(l), "text", Some("address-line1"))}
                                    {field("cislo_popisne", k::form_cislo_popisne(l), "text", None)}
                                    {field("mesto", k::form_mesto(l), "text", Some("address-level2"))}
                                    {field("psc", k::form_psc(l), "text", Some("postal-code"))}
                                </div>
                            </InquiryForm>
                        </section>
                        <section class="cenik-section" id="cenik">
                            <h2 class="cenik-main-title">{k::cenik_main_title(l)}</h2>
                            {price_group(k::cenik_stavebni_title(l), price_table(c.stavebni, 3, None, false))}
                            {price_group(k::cenik_palivove_volne_title(l), firewood_groups(c.palivove_volne, per_m3))}
                            {price_group(k::cenik_palivove_skladane_title(l), firewood_groups(c.palivove_skladane, per_m3))}
                            {price_group(k::cenik_zbytky_title(l), price_table(c.zbytky, 2, None, true))}
                            {price_group(k::cenik_kamenivo_title(l), view! {
                                <table class="cenik-table">
                                    <tbody>
                                        <tr>
                                            {match c.kamenivo {
                                                Some(price) => view! { <td style="text-align: right; padding-right: 30px;" inner_html=price.into_string()></td> }.into_any(),
                                                None => view! { <td style="text-align: right; padding-right: 30px;">{k::cenik_kamenivo_default(l)}</td> }.into_any(),
                                            }}
                                        </tr>
                                    </tbody>
                                </table>
                            })}
                            <h2 class="cenik-main-title" id="cenik-sluzeb" style="margin-top: 80px; margin-bottom: 20px;">{k::cenik_sluzeb_title(l)}</h2>
                            <div class="cenik-sluzby-container">
                                {c.sluzby.into_iter().map(|s| view! {
                                    <div class="cenik-sluzba-item">
                                        <h4 class="cenik-sluzba-label" inner_html=s.name.into_string()></h4>
                                        <div class="cenik-sluzba-box" inner_html=s.price.into_string()></div>
                                    </div>
                                }).collect_view()}
                            </div>
                        </section>
                    }
                })}
            </main>
        </Layout>
    }
}
