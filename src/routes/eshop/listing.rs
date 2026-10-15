//! GET /eshop?category=&wood=&sort=: the product grid.

use axum::extract::State;
use minijinja::context;

use super::super::Result;
use super::super::components::ProductCard;
use crate::AppState;
use crate::models::catalog::{self, CATEGORIES, CatalogProduct};
use crate::models::upload::{self, Owner};
use crate::request::Req;
use crate::views::{self, Page};

/// The default order, and the "bestsellers" order, by product key.
const RECOMMENDED: [&str; 10] = [
    "smrk",
    "dub",
    "buk",
    "late",
    "tramy",
    "fosny",
    "stepka",
    "odkory",
    "piliny",
    "okrasne_kamenivo",
];
const BESTSELLERS: [&str; 10] = [
    "buk",
    "dub",
    "smrk",
    "tramy",
    "fosny",
    "late",
    "piliny",
    "odkory",
    "stepka",
    "okrasne_kamenivo",
];

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let param = |name: &str| req.query_param(name).unwrap_or_default();
    let (category_param, wood, sort) = (param("category"), param("wood"), param("sort"));
    let category = if CATEGORIES.contains(&category_param.as_str()) {
        category_param.as_str()
    } else {
        "palivove"
    };

    let mut products: Vec<CatalogProduct> = catalog::active_products(&state.db)
        .await?
        .into_iter()
        .filter(|product| product.category == category)
        .filter(|product| match wood.as_str() {
            "tvrde" => product.hardness.as_deref() == Some("hard"),
            "mekke" => product.hardness.as_deref() == Some("soft"),
            _ => true,
        })
        .collect();
    let order: &[&str] = if sort == "nejprodavanejsi" {
        &BESTSELLERS
    } else {
        &RECOMMENDED
    };
    // A stable sort, like Ruby's sort_by on these keys.
    products.sort_by_key(|product| order.iter().position(|key| *key == product.key).unwrap_or(999));

    let money = state.rates.money(req.locale).await;
    let starting_prices = catalog::starting_prices(&state.db).await?;
    let images = upload::images_of(&state.db, Owner::Product).await?;
    let price_from = state.translations.text(req.locale, "common.price_from", &[]);
    let cards: Vec<ProductCard> = products
        .iter()
        .map(|product| ProductCard::new(product, &starting_prices, &images, &money, &price_from))
        .collect();

    let page = Page::meta(&state, &req, &format!("eshop_{category}"));
    let palivove = category_param.is_empty() || category_param == "palivove";
    Ok(views::render(
        &state,
        &req,
        "home/eshop.html",
        page,
        context! { cards, category_param, wood, sort, palivove },
    )
    .await)
}
