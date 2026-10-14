//! The e-shop: products (`catalog_products`) and the variants you can buy
//! (`catalog_variants`: a length, an amount, a grade... with a price).

use chrono::NaiveDateTime;

use super::{parameterize, tr};
use crate::db::Db;
use crate::locale::Locale;

pub const CATEGORIES: [&str; 4] = ["palivove", "rezivo", "zbytky", "kamenivo"];

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct CatalogProduct {
    pub id: i64,
    pub key: String,
    pub title: String,
    pub title_de: Option<String>,
    pub type_label: Option<String>,
    pub type_label_de: Option<String>,
    pub subtitle: Option<String>,
    pub subtitle_de: Option<String>,
    pub description: Option<String>,
    pub description_de: Option<String>,
    pub drying_note: Option<String>,
    pub drying_note_de: Option<String>,
    pub image_alt: Option<String>,
    pub image_alt_de: Option<String>,
    /// How the product page looks: "firewood", "lumber" or "simple_variant".
    pub template: String,
    pub category: String,
    pub hardness: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl CatalogProduct {
    pub fn title(&self, locale: Locale) -> String {
        tr(locale, &Some(self.title.clone()), &self.title_de)
    }
    pub fn type_label(&self, locale: Locale) -> String {
        tr(locale, &self.type_label, &self.type_label_de)
    }
    pub fn subtitle(&self, locale: Locale) -> String {
        tr(locale, &self.subtitle, &self.subtitle_de)
    }
    pub fn description(&self, locale: Locale) -> String {
        tr(locale, &self.description, &self.description_de)
    }
    pub fn drying_note(&self, locale: Locale) -> String {
        tr(locale, &self.drying_note, &self.drying_note_de)
    }
    pub fn image_alt(&self, locale: Locale) -> String {
        tr(locale, &self.image_alt, &self.image_alt_de)
    }
}

const PRODUCT_COLUMNS: &str = "id, key, title, title_de, type_label, type_label_de, subtitle, subtitle_de,
    description, description_de, drying_note, drying_note_de, image_alt, image_alt_de, template, category,
    hardness, updated_at";

/// Products shown in the shop, in the admin's order.
pub async fn active_products(db: &Db) -> sqlx::Result<Vec<CatalogProduct>> {
    sqlx::query_as(&format!(
        "SELECT {PRODUCT_COLUMNS} FROM catalog_products WHERE active = 1 ORDER BY position, id"
    ))
    .fetch_all(db)
    .await
}

pub async fn active_product(db: &Db, key: &str) -> sqlx::Result<Option<CatalogProduct>> {
    sqlx::query_as(&format!(
        "SELECT {PRODUCT_COLUMNS} FROM catalog_products WHERE active = 1 AND key = ?"
    ))
    .bind(key)
    .fetch_optional(db)
    .await
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct CatalogVariant {
    pub id: i64,
    pub catalog_product_id: i64,
    pub key: String,
    /// Firewood: "kontejner" (container) or "bedny" (crates).
    pub variant_group: Option<String>,
    pub length_label: Option<String>,
    /// Firewood: the amount in PRM.
    pub amount_value: Option<i64>,
    pub price_czk: i64,
    pub in_stock: bool,
    pub stacked: bool,
    pub grade: Option<String>,
    pub grade_de: Option<String>,
    pub width_mm: Option<i64>,
    pub height_mm: Option<i64>,
    pub variant_label: Option<String>,
    pub variant_label_de: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl CatalogVariant {
    /// Firewood neatly stacked ("rovnané") rather than loose ("sypané").
    /// Crates are always stacked; in a container it depends on the wood.
    pub fn is_stacked(&self) -> bool {
        self.variant_group.as_deref() == Some("bedny") || self.stacked
    }

    pub fn grade(&self, locale: Locale) -> String {
        tr(locale, &self.grade, &self.grade_de)
    }

    pub fn variant_label(&self, locale: Locale) -> String {
        tr(locale, &self.variant_label, &self.variant_label_de)
    }

    /// "100x50" for the dimension filter on lumber pages.
    pub fn dimension_filter_key(&self) -> Option<String> {
        Some(format!("{}x{}", self.width_mm?, self.height_mm?))
    }

    pub fn grade_filter_key(&self) -> Option<String> {
        self.grade
            .as_deref()
            .filter(|grade| !grade.trim().is_empty())
            .map(parameterize)
    }
}

const VARIANT_COLUMNS: &str = "id, catalog_product_id, key, variant_group, length_label, amount_value, price_czk,
    in_stock, stacked, grade, grade_de, width_mm, height_mm, variant_label, variant_label_de, updated_at";

/// Every variant, in the admin's order (grouped per product by the caller).
pub async fn all_variants(db: &Db) -> sqlx::Result<Vec<CatalogVariant>> {
    sqlx::query_as(&format!(
        "SELECT {VARIANT_COLUMNS} FROM catalog_variants ORDER BY position, id"
    ))
    .fetch_all(db)
    .await
}

pub async fn variants_of(db: &Db, product_id: i64) -> sqlx::Result<Vec<CatalogVariant>> {
    sqlx::query_as(&format!(
        "SELECT {VARIANT_COLUMNS} FROM catalog_variants WHERE catalog_product_id = ? ORDER BY position, id"
    ))
    .bind(product_id)
    .fetch_all(db)
    .await
}

/// "From" price of a product: its cheapest variant in stock, or the cheapest
/// of all when none is in stock.
pub fn starting_price_czk(variants: &[&CatalogVariant]) -> Option<i64> {
    variants
        .iter()
        .filter(|variant| variant.in_stock)
        .map(|variant| variant.price_czk)
        .min()
        .or_else(|| variants.iter().map(|variant| variant.price_czk).min())
}

/// The container grid on a firewood page: lengths across, amounts down.
///
/// Loose wood (25, 33 cm) and stacked wood (50 cm, 1 m) have different top
/// amounts (14 and 13 PRM), so rows are matched by position, not by value:
/// the n-th amount of each kind of wood shares a row, labelled with its
/// amounts ("5", "10", "13 a 14").
pub struct FirewoodGrid<'a> {
    variants: Vec<&'a CatalogVariant>,
    /// (stacked?, its amounts ascending), in order of first appearance.
    amounts_by_kind: Vec<(bool, Vec<i64>)>,
}

impl<'a> FirewoodGrid<'a> {
    pub fn new(variants: Vec<&'a CatalogVariant>) -> Self {
        let mut amounts_by_kind: Vec<(bool, Vec<i64>)> = Vec::new();
        for variant in &variants {
            let kind = variant.is_stacked();
            let index = match amounts_by_kind.iter().position(|(k, _)| *k == kind) {
                Some(index) => index,
                None => {
                    amounts_by_kind.push((kind, Vec::new()));
                    amounts_by_kind.len() - 1
                }
            };
            if let Some(amount) = variant.amount_value
                && !amounts_by_kind[index].1.contains(&amount)
            {
                amounts_by_kind[index].1.push(amount);
            }
        }
        for (_, amounts) in &mut amounts_by_kind {
            amounts.sort_unstable();
        }
        Self {
            variants,
            amounts_by_kind,
        }
    }

    pub fn lengths(&self) -> Vec<String> {
        let mut lengths: Vec<String> = Vec::new();
        for variant in &self.variants {
            let length = variant.length_label.clone().unwrap_or_default();
            if !lengths.contains(&length) {
                lengths.push(length);
            }
        }
        lengths
    }

    /// One entry per row: the amounts in it, ascending, e.g. [13, 14].
    pub fn rows(&self) -> Vec<Vec<i64>> {
        let row_count = self
            .amounts_by_kind
            .iter()
            .map(|(_, amounts)| amounts.len())
            .max()
            .unwrap_or(0);
        (0..row_count)
            .map(|index| {
                let mut row: Vec<i64> = self
                    .amounts_by_kind
                    .iter()
                    .filter_map(|(_, amounts)| amounts.get(index).copied())
                    .collect();
                row.sort_unstable();
                row.dedup();
                row
            })
            .collect()
    }

    /// The variant of this length in the given row, if there is one.
    pub fn variant_at(&self, row_index: usize, length: &str) -> Option<&'a CatalogVariant> {
        self.variants.iter().copied().find(|variant| {
            variant.length_label.as_deref().unwrap_or_default() == length
                && self
                    .amounts_by_kind
                    .iter()
                    .find(|(kind, _)| *kind == variant.is_stacked())
                    .and_then(|(_, amounts)| amounts.get(row_index))
                    .copied()
                    == variant.amount_value
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variant(length: &str, amount: i64, stacked: bool, price: i64, in_stock: bool) -> CatalogVariant {
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

    #[test]
    fn loose_and_stacked_top_amounts_share_the_last_row() {
        let variants = [
            variant("25 cm", 5, false, 1, true),
            variant("25 cm", 14, false, 1, true),
            variant("50 cm", 5, true, 1, true),
            variant("50 cm", 13, true, 1, true),
        ];
        let grid = FirewoodGrid::new(variants.iter().collect());
        assert_eq!(grid.lengths(), ["25 cm", "50 cm"]);
        assert_eq!(grid.rows(), vec![vec![5], vec![13, 14]]);
        assert_eq!(grid.variant_at(1, "25 cm").and_then(|v| v.amount_value), Some(14));
        assert_eq!(grid.variant_at(1, "50 cm").and_then(|v| v.amount_value), Some(13));
        assert!(grid.variant_at(2, "50 cm").is_none());
    }

    #[test]
    fn starting_price_prefers_variants_in_stock() {
        let cheap_sold_out = variant("25 cm", 5, false, 100, false);
        let in_stock = variant("25 cm", 10, false, 200, true);
        assert_eq!(starting_price_czk(&[&cheap_sold_out, &in_stock]), Some(200));
        assert_eq!(starting_price_czk(&[&cheap_sold_out]), Some(100));
        assert_eq!(starting_price_czk(&[]), None);
    }
}
