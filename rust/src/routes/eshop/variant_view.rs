//! A variant as the product page templates show it.

use std::collections::HashMap;

use serde::Serialize;

use crate::models::catalog::CatalogVariant;
use crate::models::upload::Upload;
use crate::money::Money;

#[derive(Serialize)]
pub struct VariantView {
    pub id: i64,
    pub in_stock: bool,
    pub price: String,
    pub image: Option<String>,
    pub stacked: bool,
    pub length: String,
    pub amount: Option<i64>,
    pub grade: Option<String>,
    pub width_mm: Option<i64>,
    pub height_mm: Option<i64>,
    pub label: String,
    /// The value the lumber filter buttons match ("200x50", "jakost-ab").
    pub filter_value: String,
}

/// Turns variants into [`VariantView`]s: prices in the page's currency,
/// the variant's own photo or a fallback.
pub struct VariantViews {
    pub money: Money,
    pub images: HashMap<i64, Upload>,
}

impl VariantViews {
    pub fn view(&self, variant: &CatalogVariant, fallback_image: Option<String>, filter_value: String) -> VariantView {
        let locale = self.money.locale;
        VariantView {
            id: variant.id,
            in_stock: variant.in_stock,
            price: self.money.display(variant.price_czk),
            image: self.images.get(&variant.id).map(Upload::url).or(fallback_image),
            stacked: variant.is_stacked(),
            length: variant.length_label.clone().unwrap_or_default(),
            amount: variant.amount_value,
            grade: variant
                .grade
                .as_deref()
                .filter(|grade| !grade.trim().is_empty())
                .map(|_| variant.grade(locale)),
            width_mm: variant.width_mm,
            height_mm: variant.height_mm,
            label: variant.variant_label(locale),
            filter_value,
        }
    }
}
