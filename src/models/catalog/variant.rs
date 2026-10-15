//! A variant you can put in the cart (`catalog_variants`): a container of
//! firewood, a length of beam, a load of wood chips...

use chrono::NaiveDateTime;

use crate::db::Db;
use crate::locale::Locale;
use crate::models::{parameterize, tr};

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

pub async fn variants_of(db: &Db, product_id: i64) -> sqlx::Result<Vec<CatalogVariant>> {
    sqlx::query_as(&format!(
        "SELECT {VARIANT_COLUMNS} FROM catalog_variants WHERE catalog_product_id = ? ORDER BY position, id"
    ))
    .bind(product_id)
    .fetch_all(db)
    .await
}

pub async fn find(db: &Db, id: i64) -> sqlx::Result<Option<CatalogVariant>> {
    sqlx::query_as(&format!("SELECT {VARIANT_COLUMNS} FROM catalog_variants WHERE id = ?"))
        .bind(id)
        .fetch_optional(db)
        .await
}
