//! Catalogue products (`catalog_products`): the e-shop's products, each
//! shown with one of three page layouts.

use std::path::Path;

use super::messages::{blank, invalid, taken};
use super::record::{Table, Values, boolean, integer, text};
use super::variants;
use crate::db::Db;
use crate::forms::Errors;
use crate::models::upload::Owner;
use crate::storage;

pub const TEMPLATES: [&str; 3] = ["firewood", "lumber", "simple_variant"];
pub const CATEGORIES: [&str; 4] = ["palivove", "rezivo", "zbytky", "kamenivo"];
pub const HARDNESS: [&str; 2] = ["hard", "soft"];

pub const TABLE: Table = Table {
    name: "catalog_products",
    columns: &[
        text("key"),
        text("template"),
        text("category"),
        text("hardness"),
        boolean("active", true),
        integer("position", "Pořadí v katalogu", Some("0")),
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

/// The fields the form edits (all columns; the image comes separately).
pub const FORM_FIELDS: [&str; 18] = [
    "key",
    "template",
    "category",
    "hardness",
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

/// An empty choice of hardness means none.
pub fn normalize(values: &mut Values) {
    if !values.is_present("hardness") {
        values.set("hardness", None);
    }
}

/// Where the record's photo is stored.
pub fn image(id: i64) -> (Owner, i64) {
    (Owner::Product, id)
}

/// The product's checks. `id` is the product being edited (None for a new
/// one).
pub async fn validate(db: &Db, values: &Values, id: Option<i64>) -> sqlx::Result<Errors> {
    let mut errors = Errors::new();
    match values.get("key") {
        Some(key) if !key.trim().is_empty() => {
            let existing: Option<i64> =
                sqlx::query_scalar("SELECT id FROM catalog_products WHERE key = ? AND id IS NOT ?")
                    .bind(key)
                    .bind(id)
                    .fetch_optional(db)
                    .await?;
            if existing.is_some() {
                errors.add("key", taken("Klíč"));
            }
        }
        _ => errors.add("key", blank("Klíč")),
    }
    if !TEMPLATES.contains(&values.get("template").unwrap_or_default()) {
        errors.add("template", invalid("Šablona zobrazení"));
    }
    if !CATEGORIES.contains(&values.get("category").unwrap_or_default()) {
        errors.add("category", invalid("Kategorie filtru"));
    }
    if let Some(hardness) = values.get("hardness")
        && !HARDNESS.contains(&hardness)
    {
        errors.add("hardness", invalid("Tvrdost dřeva"));
    }
    TABLE.check_numbers(values, &mut errors);
    Ok(errors)
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
    storage::detach(db, storage_root, Owner::Product, id).await?;
    TABLE.delete(db, id).await?;
    Ok(())
}
