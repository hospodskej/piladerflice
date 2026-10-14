//! The home page, Kontakt (with the price list and quote form), Služby and
//! the Sortiment overview.

use std::collections::HashMap;

use axum::extract::State;
use chrono::Utc;
use minijinja::context;
use serde::Serialize;

use super::Result;
use super::catalog::{ProductCard, local_business_schema};
use crate::AppState;
use crate::models::catalog;
use crate::models::content;
use crate::models::pricelist::{self, INQUIRY_CATEGORIES, pricelist_html};
use crate::models::uploads;
use crate::request::Req;
use crate::views::{self, Page};

/// The products featured as bestsellers on the home page, in this order.
const BESTSELLER_KEYS: [&str; 3] = ["akat", "tramy", "dub"];

fn t(state: &AppState, req: &Req, key: &str) -> String {
    state.translations.text(req.locale, key, &[])
}

/// The JSON-LD business data for <head>.
fn business_schema(state: &AppState, req: &Req) -> String {
    local_business_schema(
        req,
        &t(state, req, "site.name"),
        &t(state, req, "meta.home.description"),
        &format!("{}{}", req.base_url, state.assets.url("logo-wm.webp")),
        [
            t(state, req, "meta.area_served.south_moravia"),
            t(state, req, "meta.area_served.north_austria"),
        ],
    )
}

#[derive(Serialize)]
struct PromoView {
    title: String,
    features: [String; 3],
    price: String,
    link: String,
    image: Option<String>,
}

#[derive(Serialize)]
struct ServiceItem {
    title: String,
    /// Trusted HTML typed in the admin (it has <br> line breaks).
    content: String,
    button_text: Option<String>,
    button_path: Option<String>,
}

#[derive(Serialize)]
struct Slide {
    title: String,
    content: String,
    image: Option<String>,
}

#[derive(Serialize)]
struct ReviewView {
    name: String,
    avatar: Option<String>,
    initial: String,
    color: &'static str,
    stars: String,
    time_ago: String,
    text: String,
}

#[derive(Serialize)]
struct Tile {
    title: String,
    text: String,
    image: Option<String>,
    link: String,
}

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let locale = req.locale;
    let money = state.rates.money(locale).await;
    let today = Utc::now().date_naive();

    let promo = match content::weekly_promo(&state.db, today).await? {
        Some(promo) => {
            let images = uploads::images_of(&state.db, "Promo").await?;
            Some(PromoView {
                title: promo.title(locale),
                features: promo.features(locale),
                price: money.convert_text(&promo.price(locale)),
                link: req.path_for(promo.link.as_deref().unwrap_or("/")),
                image: images.get(&promo.id).map(uploads::Upload::url),
            })
        }
        None => None,
    };

    let products = catalog::active_products(&state.db).await?;
    let variants = catalog::all_variants(&state.db).await?;
    let images = uploads::images_of(&state.db, "CatalogProduct").await?;
    let price_from = t(&state, &req, "common.price_from");
    let bestsellers: Vec<ProductCard> = BESTSELLER_KEYS
        .iter()
        .filter_map(|key| products.iter().find(|product| product.key == *key))
        .map(|product| ProductCard::new(product, &variants, &images, &money, &price_from))
        .collect();

    let tiles: Vec<Tile> = content::category_tiles(&state.db)
        .await?
        .iter()
        .map(|tile| Tile {
            title: tile.title(locale),
            text: tile.text(locale),
            image: tile.image.clone(),
            link: req.path_for(tile.link.as_deref().unwrap_or("/")),
        })
        .collect();

    let services: Vec<ServiceItem> = content::services(&state.db)
        .await?
        .iter()
        .map(|service| {
            let has_button = service
                .button_text
                .as_deref()
                .is_some_and(|text| !text.trim().is_empty())
                && service
                    .button_path
                    .as_deref()
                    .is_some_and(|path| !path.trim().is_empty());
            ServiceItem {
                title: service.title(locale),
                content: service.content(locale),
                button_text: has_button.then(|| service.button_text(locale)),
                button_path: has_button.then(|| req.path_for(service.button_path.as_deref().unwrap_or_default())),
            }
        })
        .collect();

    let slides: Vec<Slide> = content::faq_items(&state.db)
        .await?
        .iter()
        .map(|item| Slide {
            title: item.title(locale),
            content: item.content(locale),
            image: item.image.clone(),
        })
        .collect();

    let reviews: Vec<ReviewView> = content::reviews(&state.db)
        .await?
        .iter()
        .map(|review| ReviewView {
            name: review.name.clone(),
            avatar: review.avatar.clone().filter(|avatar| !avatar.trim().is_empty()),
            initial: review.initial(),
            color: review.avatar_color(),
            stars: "★".repeat(review.stars.max(0) as usize),
            time_ago: review.time_ago(locale, today),
            text: review.text.clone(),
        })
        .collect();

    let mut page = Page::meta(&state, &req, "home").brand_only_title(&state, &req, "home");
    page.head_html = Some(business_schema(&state, &req));
    Ok(views::render(
        &state,
        &req,
        "home/index.html",
        page,
        context! { promo, bestsellers, tiles, services, slides, reviews },
    ))
}

#[derive(Serialize)]
struct PriceRow {
    name: String,
    details: String,
    price: String,
    /// "---" means "no price", shown without the "/ PRM" suffix.
    has_price: bool,
}

#[derive(Serialize)]
struct PriceGroup {
    name: String,
    items: Vec<PriceRow>,
}

#[derive(Serialize)]
struct FormSection {
    category: &'static str,
    fields: Vec<FormField>,
}

#[derive(Serialize)]
struct FormField {
    name: &'static str,
    options: Vec<String>,
    /// Offer "other..." with a free text field.
    custom: bool,
    /// Quantity of lumber: a number field instead of a list.
    number: bool,
}

pub async fn kontakt(State(state): State<AppState>, req: Req) -> Result {
    let locale = req.locale;
    let money = state.rates.money(locale).await;

    let mut by_category: HashMap<String, Vec<pricelist::PricelistItem>> = HashMap::new();
    for item in pricelist::pricelist(&state.db).await? {
        by_category
            .entry(item.category.clone().unwrap_or_default())
            .or_default()
            .push(item);
    }
    let row = |item: &pricelist::PricelistItem| {
        let price = item.price(&money);
        PriceRow {
            name: pricelist_html(&item.item_name(locale)),
            details: pricelist_html(&item.details(locale)),
            has_price: price != "---",
            price: pricelist_html(&price),
        }
    };
    let rows = |category: &str| -> Vec<PriceRow> {
        by_category
            .get(category)
            .map_or(Vec::new(), |items| items.iter().map(row).collect())
    };
    let groups = |category: &str| -> Vec<PriceGroup> {
        let mut groups: Vec<PriceGroup> = Vec::new();
        for item in by_category.get(category).into_iter().flatten() {
            let name = pricelist_html(&item.subcategory(locale));
            match groups.iter_mut().find(|group| group.name == name) {
                Some(group) => group.items.push(row(item)),
                None => groups.push(PriceGroup {
                    name,
                    items: vec![row(item)],
                }),
            }
        }
        groups
    };
    let kamenivo = by_category
        .get("kamenivo")
        .and_then(|items| items.first())
        .map(|item| pricelist_html(&item.price(&money)));

    let options = pricelist::inquiry_options(&state.db).await?;
    let form_sections: Vec<FormSection> = INQUIRY_CATEGORIES
        .iter()
        .map(|category| FormSection {
            category,
            fields: pricelist::inquiry_row_fields(category)
                .iter()
                .map(|field| FormField {
                    name: field,
                    options: options
                        .iter()
                        .filter(|option| option.category == *category && option.field == *field)
                        .map(|option| option.value(locale))
                        .collect(),
                    custom: *category == "stavebni" || *field == "mnozstvi",
                    number: *category == "stavebni" && *field == "mnozstvi",
                })
                .collect(),
        })
        .collect();

    let mut page = Page::meta(&state, &req, "kontakt");
    page.head_html = Some(business_schema(&state, &req));
    Ok(views::render(
        &state,
        &req,
        "home/kontakt.html",
        page,
        context! {
            stavebni => rows("stavebni"),
            palivove_volne => groups("palivove_volne"),
            palivove_skladane => groups("palivove_skladane"),
            zbytky => rows("zbytky"),
            kamenivo,
            sluzby_cenik => rows("sluzby"),
            form_sections,
        },
    ))
}

#[derive(Serialize)]
struct ServiceSection {
    anchor: String,
    title: String,
    content: String,
    button_text: Option<String>,
    images: Vec<String>,
}

pub async fn sluzby(State(state): State<AppState>, req: Req) -> Result {
    let locale = req.locale;
    let sections: Vec<ServiceSection> = content::services_page(&state.db)
        .await?
        .iter()
        .map(|service| ServiceSection {
            anchor: service_anchor(
                service.button_path.as_deref().unwrap_or_default(),
                service.title.as_deref().unwrap_or_default(),
            ),
            title: service.title(locale),
            content: simple_format(&service.content(locale)),
            button_text: service
                .button_text
                .as_deref()
                .is_some_and(|text| !text.trim().is_empty())
                .then(|| service.button_text(locale)),
            images: service.image_list(),
        })
        .collect();
    let page = Page::meta(&state, &req, "sluzby");
    Ok(views::render(
        &state,
        &req,
        "home/sluzby.html",
        page,
        context! { sections },
    ))
}

/// "/sluzby#doprava" -> "doprava"; without an anchor, the title as a slug
/// (Rails: `button_path.split('#').last.presence || title.parameterize`).
fn service_anchor(button_path: &str, title: &str) -> String {
    match button_path.split('#').rfind(|part| !part.is_empty()) {
        Some(last) if !button_path.is_empty() => last.to_string(),
        _ => crate::models::parameterize(title),
    }
}

/// Rails' `simple_format`: the text made safe, blank lines become
/// paragraphs and single line breaks become <br>.
fn simple_format(text: &str) -> String {
    let clean = ammonia::clean(text);
    let normalized = clean.replace("\r\n", "\n").replace('\r', "\n");
    let paragraphs: Vec<String> = normalized
        .split("\n\n")
        .map(|paragraph| paragraph.trim_matches('\n'))
        .filter(|paragraph| !paragraph.trim().is_empty())
        .map(|paragraph| format!("<p>{}</p>", paragraph.replace('\n', "\n<br />")))
        .collect();
    if paragraphs.is_empty() {
        "<p></p>".to_string()
    } else {
        paragraphs.join("\n\n")
    }
}

pub async fn sortiment(State(state): State<AppState>, req: Req) -> Result {
    let tiles: Vec<Tile> = content::category_tiles(&state.db)
        .await?
        .iter()
        .map(|tile| Tile {
            title: tile.title(req.locale),
            text: tile.text(req.locale),
            image: tile.image.clone(),
            link: req.path_for(tile.link.as_deref().unwrap_or("/")),
        })
        .collect();
    let page = Page::meta(&state, &req, "sortiment");
    Ok(views::render(
        &state,
        &req,
        "home/sortiment.html",
        page,
        context! { tiles },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_anchors() {
        assert_eq!(service_anchor("/sluzby#doprava", "Doprava"), "doprava");
        assert_eq!(service_anchor("", "Pořez / Prodej kulatiny"), "porez-prodej-kulatiny");
        assert_eq!(service_anchor("/sluzby", "X"), "/sluzby");
    }

    #[test]
    fn simple_format_makes_paragraphs() {
        assert_eq!(simple_format("a\nb\n\nc"), "<p>a\n<br />b</p>\n\n<p>c</p>");
        assert_eq!(simple_format("x <br /> y"), "<p>x <br> y</p>");
        assert_eq!(simple_format("<script>alert(1)</script>ok"), "<p>ok</p>");
    }
}
