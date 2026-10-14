//! A product of the e-shop (`catalog_products`).

use chrono::NaiveDateTime;

use crate::db::Db;
use crate::locale::Locale;
use crate::models::tr;

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
