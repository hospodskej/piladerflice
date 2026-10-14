//! The e-shop: products (`catalog_products`), the variants you can buy
//! (`catalog_variants`: a length, an amount, a grade... with a price) and
//! the firewood container grid.

mod firewood_grid;
mod product;
mod variant;

pub use firewood_grid::FirewoodGrid;
pub use product::{CATEGORIES, CatalogProduct, active_product, active_products};
pub use variant::{CatalogVariant, all_variants, starting_price_czk, variants_of};

/// A variant to build test data from.
#[cfg(test)]
pub mod test_data {
    use chrono::NaiveDateTime;

    use super::CatalogVariant;

    pub fn variant(length: &str, amount: i64, stacked: bool, price: i64, in_stock: bool) -> CatalogVariant {
        CatalogVariant {
            id: amount,
            catalog_product_id: 1,
            key: format!("{length}-{amount}"),
            variant_group: Some("kontejner".into()),
            length_label: Some(length.into()),
            amount_value: Some(amount),
            price_czk: price,
            in_stock,
            stacked,
            grade: None,
            grade_de: None,
            width_mm: None,
            height_mm: None,
            variant_label: None,
            variant_label_de: None,
            updated_at: NaiveDateTime::default(),
        }
    }
}
