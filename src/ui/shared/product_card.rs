//! A product tile: in the e-shop grid, the home page bestsellers and the
//! recommendations under a product.

use std::collections::HashMap;

use leptos::prelude::*;
use serde::Serialize;

use crate::models::catalog::CatalogProduct;
use crate::models::upload::Upload;
use crate::money::Money;
use crate::ui::{path, t};

/// A product tile: picture, name, kind and "from" price.
#[derive(Serialize)]
pub struct ProductCard {
    pub key: String,
    pub title: String,
    pub type_label: String,
    pub image: Option<String>,
    /// "od 1 500 Kč", or None when the product has no variants yet.
    pub price_from: Option<String>,
}

impl ProductCard {
    pub fn new(
        product: &CatalogProduct,
        starting_prices: &HashMap<i64, i64>,
        images: &HashMap<i64, Upload>,
        money: &Money,
        price_from_label: &str,
    ) -> Self {
        Self {
            key: product.key.clone(),
            title: product.title(money.locale),
            type_label: product.type_label(money.locale),
            image: images.get(&product.id).map(Upload::url),
            price_from: starting_prices
                .get(&product.id)
                .map(|&czk| format!("{price_from_label} {}", money.display(czk))),
        }
    }
}

/// The tile in the e-shop grid and the recommendations. `badge` numbers the
/// top products (1., 2., 3.).
#[component]
pub fn ProductCardView(product: ProductCard, #[prop(default = None)] badge: Option<usize>) -> impl IntoView {
    let href = path(&format!("/eshop/{}", product.key));
    view! {
        <div class="product-card">
            <a class="product-image-wrapper-eshop" href=href.clone()>
                {badge.map(|badge| view! { <span class="product-badge">{format!("{badge}.")}</span> })}
                {product.image.map(|src| view! { <img alt=product.title.clone() class="product-img" src=src/> })}
            </a>
            <div class="product-info-eshop">
                <h4 class="product-title">{product.title}</h4>
                <p class="product-type">{product.type_label}</p>
                <p class="product-stock">{t("common.stock")}</p>
                <p class="product-price">{product.price_from.unwrap_or_else(|| "---".into())}</p>
                <a class="btn-primary btn-detail" href=href>{t("common.detail")}</a>
            </div>
        </div>
    }
}
