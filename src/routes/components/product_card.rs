//! A product tile: in the e-shop grid, the home page bestsellers and the
//! recommendations under a product.

use std::collections::HashMap;

use serde::Serialize;

use crate::models::catalog::{self, CatalogProduct, CatalogVariant};
use crate::models::upload::Upload;
use crate::money::Money;

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
        variants: &[CatalogVariant],
        images: &HashMap<i64, Upload>,
        money: &Money,
        price_from_label: &str,
    ) -> Self {
        let own: Vec<&CatalogVariant> = variants.iter().filter(|v| v.catalog_product_id == product.id).collect();
        Self {
            key: product.key.clone(),
            title: product.title(money.locale),
            type_label: product.type_label(money.locale),
            image: images.get(&product.id).map(Upload::url),
            price_from: catalog::starting_price_czk(&own)
                .map(|czk| format!("{price_from_label} {}", money.display(czk))),
        }
    }
}
