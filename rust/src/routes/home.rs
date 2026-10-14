//! The home page: weekly promo, bestsellers, categories, services, the
//! "might interest you" slides and Google reviews.

use axum::extract::State;
use chrono::{NaiveDate, Utc};
use minijinja::context;
use serde::Serialize;

use super::Result;
use super::components::{CategoryTile, ProductCard, business_schema, category_tile};
use crate::AppState;
use crate::models::upload::{self, Upload};
use crate::models::{catalog, content};
use crate::money::Money;
use crate::request::Req;
use crate::views::{self, Page};

/// The products featured as bestsellers, in this order.
const BESTSELLER_KEYS: [&str; 3] = ["akat", "tramy", "dub"];

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let money = state.rates.money(req.locale).await;
    let today = Utc::now().date_naive();

    let promo = promo(&state, &req, &money, today).await?;
    let bestsellers = bestsellers(&state, &money).await?;
    let tiles: Vec<CategoryTile> = category_tile::all(&state, &req).await?;
    let services = services(&state, &req).await?;
    let slides = slides(&state, &req).await?;
    let reviews = reviews(&state, &req, today).await?;

    let mut page = Page::meta(&state, &req, "home").brand_only_title(&state, &req, "home");
    page.head_html = Some(business_schema::tag(&state, &req));
    Ok(views::render(
        &state,
        &req,
        "home/index.html",
        page,
        context! { promo, bestsellers, tiles, services, slides, reviews },
    ))
}

#[derive(Serialize)]
struct PromoView {
    title: String,
    features: [String; 3],
    price: String,
    link: String,
    image: Option<String>,
}

/// This week's promo; its price in euros for Austria.
async fn promo(state: &AppState, req: &Req, money: &Money, today: NaiveDate) -> Result<Option<PromoView>> {
    let Some(promo) = content::weekly_promo(&state.db, today).await? else {
        return Ok(None);
    };
    let images = upload::images_of(&state.db, "Promo").await?;
    Ok(Some(PromoView {
        title: promo.title(req.locale),
        features: promo.features(req.locale),
        price: money.convert_text(&promo.price(req.locale)),
        link: req.path_for(promo.link.as_deref().unwrap_or("/")),
        image: images.get(&promo.id).map(Upload::url),
    }))
}

async fn bestsellers(state: &AppState, money: &Money) -> Result<Vec<ProductCard>> {
    let products = catalog::active_products(&state.db).await?;
    let variants = catalog::all_variants(&state.db).await?;
    let images = upload::images_of(&state.db, "CatalogProduct").await?;
    let price_from = state.translations.text(money.locale, "common.price_from", &[]);
    Ok(BESTSELLER_KEYS
        .iter()
        .filter_map(|key| products.iter().find(|product| product.key == *key))
        .map(|product| ProductCard::new(product, &variants, &images, money, &price_from))
        .collect())
}

#[derive(Serialize)]
struct ServiceItem {
    title: String,
    /// Trusted HTML typed in the admin (it has <br> line breaks).
    content: String,
    button_text: Option<String>,
    button_path: Option<String>,
}

/// The services accordion; a button only when both its text and link are set.
async fn services(state: &AppState, req: &Req) -> Result<Vec<ServiceItem>> {
    let present = |value: &Option<String>| value.as_deref().is_some_and(|text| !text.trim().is_empty());
    Ok(content::services(&state.db)
        .await?
        .iter()
        .map(|service| {
            let has_button = present(&service.button_text) && present(&service.button_path);
            ServiceItem {
                title: service.title(req.locale),
                content: service.content(req.locale),
                button_text: has_button.then(|| service.button_text(req.locale)),
                button_path: has_button.then(|| req.path_for(service.button_path.as_deref().unwrap_or_default())),
            }
        })
        .collect())
}

#[derive(Serialize)]
struct Slide {
    title: String,
    content: String,
    image: Option<String>,
}

async fn slides(state: &AppState, req: &Req) -> Result<Vec<Slide>> {
    Ok(content::faq_items(&state.db)
        .await?
        .iter()
        .map(|item| Slide {
            title: item.title(req.locale),
            content: item.content(req.locale),
            image: item.image.clone(),
        })
        .collect())
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

async fn reviews(state: &AppState, req: &Req, today: NaiveDate) -> Result<Vec<ReviewView>> {
    Ok(content::reviews(&state.db)
        .await?
        .iter()
        .map(|review| ReviewView {
            name: review.name.clone(),
            avatar: review.avatar.clone().filter(|avatar| !avatar.trim().is_empty()),
            initial: review.initial(),
            color: review.avatar_color(),
            stars: "★".repeat(review.stars.max(0) as usize),
            time_ago: review.time_ago(req.locale, today),
            text: review.text.clone(),
        })
        .collect())
}
