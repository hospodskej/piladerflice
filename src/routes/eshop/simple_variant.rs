//! Simple product pages (wood chips, sawdust, gravel...): one card per
//! variant.

use crate::models::catalog::CatalogVariant;
use crate::ui::eshop::{SimpleVariants, VariantViews};

pub fn data(variants: &[CatalogVariant], views: &VariantViews, container: &str) -> SimpleVariants {
    SimpleVariants {
        cards: variants
            .iter()
            .map(|v| views.view(v, Some(container.to_string()), String::new()))
            .collect(),
    }
}
