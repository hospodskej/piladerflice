//! Simple product pages (wood chips, sawdust, gravel...): one card per
//! variant, two or three in a row.

use minijinja::{Value, context};

use super::variant_view::{VariantView, VariantViews};
use crate::models::catalog::CatalogVariant;

pub fn data(variants: &[CatalogVariant], views: &VariantViews, container: &str) -> Value {
    let cards: Vec<VariantView> = variants
        .iter()
        .map(|v| views.view(v, Some(container.to_string()), String::new()))
        .collect();
    let grid_class = if variants.len() <= 2 {
        "simple-grid-2"
    } else {
        "simple-grid-3"
    };
    context! { cards, grid_class }
}
