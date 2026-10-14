//! Lumber product pages: variant cards with filter buttons by dimension
//! (or by grade when the variants have no dimensions).

use minijinja::{Value, context};
use serde::Serialize;

use super::variant_view::{VariantView, VariantViews};
use crate::models::catalog::CatalogVariant;

#[derive(Serialize)]
struct FilterOption {
    key: String,
    label: String,
}

pub fn data(variants: &[CatalogVariant], views: &VariantViews, product_image: Option<String>) -> Value {
    let locale = views.money.locale;
    let has_dimensions = variants.iter().any(|v| v.dimension_filter_key().is_some());
    let filter_key = |v: &CatalogVariant| {
        if has_dimensions {
            v.dimension_filter_key()
        } else {
            v.grade_filter_key()
        }
    };

    // One button per distinct value, in the variants' order.
    let mut filters: Vec<FilterOption> = Vec::new();
    for variant in variants {
        let Some(key) = filter_key(variant).filter(|key| !key.is_empty()) else {
            continue;
        };
        if filters.iter().any(|option| option.key == key) {
            continue;
        }
        let label = if has_dimensions {
            format!(
                "{}×{} mm",
                variant.width_mm.unwrap_or_default(),
                variant.height_mm.unwrap_or_default()
            )
        } else {
            variant.grade(locale)
        };
        filters.push(FilterOption { key, label });
    }

    let cards: Vec<VariantView> = variants
        .iter()
        .map(|v| views.view(v, product_image.clone(), filter_key(v).unwrap_or_default()))
        .collect();
    context! { has_dimensions, filters, cards }
}
