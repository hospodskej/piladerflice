//! Catalogue products (`catalog_products`): the e-shop's products, each
//! shown with one of three page layouts.

use std::path::Path;

use super::record::{Table, Values, boolean, integer, text};
use super::variants;
use crate::db::Db;
use crate::i18n::Translations;
use crate::locale::Locale;
use crate::models::validation::{Errors, Validator};
use crate::storage::{self, Attachment};

pub const TEMPLATES: [&str; 3] = ["firewood", "lumber", "simple_variant"];
pub const CATEGORIES: [&str; 4] = ["palivove", "rezivo", "zbytky", "kamenivo"];
pub const HARDNESS: [&str; 2] = ["hard", "soft"];

pub const TABLE: Table = Table {
    name: "catalog_products",
    param: "catalog_product",
    columns: &[
        text("key"),
        text("template"),
        text("category"),
        text("hardness"),
        boolean("active", true),
        integer("position", Some("0")),
        text("title"),
        text("title_de"),
        text("type_label"),
        text("type_label_de"),
        text("subtitle"),
        text("subtitle_de"),
        text("description"),
        text("description_de"),
        text("drying_note"),
        text("drying_note_de"),
        text("image_alt"),
        text("image_alt_de"),
    ],
};

/// The fields the form may change (everything; the image comes separately).
pub const PERMITTED: [&str; 19] = [
    "key",
    "template",
    "category",
    "hardness",
    "image",
    "active",
    "position",
    "title",
    "title_de",
    "type_label",
    "type_label_de",
    "subtitle",
    "subtitle_de",
    "description",
    "description_de",
    "drying_note",
    "drying_note_de",
    "image_alt",
    "image_alt_de",
];

/// A product as the catalogue table lists it.
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Listed {
    pub id: i64,
    pub title: String,
    pub key: String,
    pub template: String,
    pub category: String,
    pub active: bool,
    pub variant_count: i64,
    /// The cheapest variant in stock, or the cheapest at all.
    pub starting_price_czk: Option<i64>,
}

pub async fn list(db: &Db) -> sqlx::Result<Vec<Listed>> {
    sqlx::query_as(
        "SELECT p.id, p.title, p.key, p.template, p.category, p.active,
           (SELECT COUNT(*) FROM catalog_variants v WHERE v.catalog_product_id = p.id) AS variant_count,
           COALESCE(
             (SELECT MIN(price_czk) FROM catalog_variants v WHERE v.catalog_product_id = p.id AND v.in_stock),
             (SELECT MIN(price_czk) FROM catalog_variants v WHERE v.catalog_product_id = p.id)
           ) AS starting_price_czk
         FROM catalog_products p ORDER BY p.position, p.id",
    )
    .fetch_all(db)
    .await
}

/// Rails' `normalizes :hardness`: an empty choice means none.
pub fn normalize(values: &mut Values) {
    if !values.is_present("hardness") {
        values.set("hardness", None);
    }
}

pub fn image(id: i64) -> Attachment<'static> {
    Attachment::new("CatalogProduct", id, "image")
}

/// The product's checks, in Rails' order. `id` is the product being
/// edited (None for a new one).
pub async fn validate(db: &Db, translations: &Translations, values: &Values, id: Option<i64>) -> sqlx::Result<Errors> {
    let mut check = Validator::new(translations, Locale::Cs, "catalog_product");
    match values.get("key") {
        Some(key) if !key.trim().is_empty() => {
            let taken: Option<i64> =
                sqlx::query_scalar("SELECT id FROM catalog_products WHERE key = ? AND id IS NOT ?")
                    .bind(key)
                    .bind(id)
                    .fetch_optional(db)
                    .await?;
            if taken.is_some() {
                check.add("key", "taken");
            }
        }
        _ => check.add("key", "blank"),
    }
    if !TEMPLATES.contains(&values.get("template").unwrap_or_default()) {
        check.add("template", "inclusion");
    }
    if !CATEGORIES.contains(&values.get("category").unwrap_or_default()) {
        check.add("category", "inclusion");
    }
    if let Some(hardness) = values.get("hardness")
        && !HARDNESS.contains(&hardness)
    {
        check.add("hardness", "inclusion");
    }
    Ok(check.finish())
}

/// Deletes the product with its variants and all their pictures.
pub async fn delete(db: &Db, storage_root: &Path, id: i64) -> anyhow::Result<()> {
    let variant_ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM catalog_variants WHERE catalog_product_id = ?")
        .bind(id)
        .fetch_all(db)
        .await?;
    for variant_id in variant_ids {
        variants::delete(db, storage_root, variant_id).await?;
    }
    storage::detach(db, storage_root, image(id)).await?;
    TABLE.delete(db, id).await?;
    Ok(())
}
