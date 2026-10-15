//! Variants (`catalog_variants`): what can actually be bought, each with
//! its own price — a length and amount of firewood, a size of beam...

use std::path::Path;

use super::record::{Table, Values, boolean, integer, text};
use crate::db::Db;
use crate::i18n::Translations;
use crate::locale::Locale;
use crate::models::upload::Owner;
use crate::models::validation::{Errors, Validator};
use crate::storage;

pub const VARIANT_GROUPS: [&str; 2] = ["kontejner", "bedny"];

pub const TABLE: Table = Table {
    name: "catalog_variants",
    param: "catalog_variant",
    columns: &[
        integer("catalog_product_id", None),
        text("key"),
        text("variant_group"),
        text("length_label"),
        text("variant_label"),
        text("variant_label_de"),
        integer("amount_value", None),
        boolean("stacked", false),
        integer("price_czk", None),
        boolean("in_stock", true),
        integer("position", Some("0")),
        text("grade"),
        text("grade_de"),
        integer("width_mm", None),
        integer("height_mm", None),
    ],
};

pub const PERMITTED: [&str; 15] = [
    "key",
    "variant_group",
    "length_label",
    "variant_label",
    "variant_label_de",
    "amount_value",
    "stacked",
    "price_czk",
    "in_stock",
    "position",
    "grade",
    "grade_de",
    "width_mm",
    "height_mm",
    "image",
];

/// Where the record's photo is stored.
pub fn image(id: i64) -> (Owner, i64) {
    (Owner::Variant, id)
}

/// The variant's checks, in Rails' order: key given and not used by another
/// variant of the same product, a price above zero, a known group.
pub async fn validate(db: &Db, translations: &Translations, values: &Values, id: Option<i64>) -> sqlx::Result<Errors> {
    let mut check = Validator::new(translations, Locale::Cs, "catalog_variant");
    match values.get("key") {
        Some(key) if !key.trim().is_empty() => {
            let taken: Option<i64> = sqlx::query_scalar(
                "SELECT id FROM catalog_variants WHERE key = ? AND catalog_product_id = ? AND id IS NOT ?",
            )
            .bind(key)
            .bind(values.integer("catalog_product_id"))
            .bind(id)
            .fetch_optional(db)
            .await?;
            if taken.is_some() {
                check.add("key", "taken");
            }
        }
        _ => check.add("key", "blank"),
    }
    // Checked as typed: "12 Kč" is not a number, though it would save as 12.
    match values
        .get("price_czk")
        .and_then(|price| price.trim().parse::<f64>().ok())
    {
        Some(price) if price > 0.0 => {}
        Some(_) => check.add_with_count("price_czk", "greater_than", Some(0)),
        None => check.add("price_czk", "not_a_number"),
    }
    if let Some(group) = values.get("variant_group")
        && !VARIANT_GROUPS.contains(&group)
    {
        check.add("variant_group", "inclusion");
    }
    Ok(check.finish())
}

/// Firewood that is neatly stacked: always in crates, in a container when
/// ticked.
pub fn is_stacked(values: &Values) -> bool {
    values.get("variant_group") == Some("bedny") || values.is_true("stacked")
}

/// Deletes the variant and its picture.
pub async fn delete(db: &Db, storage_root: &Path, id: i64) -> anyhow::Result<()> {
    storage::detach(db, storage_root, Owner::Variant, id).await?;
    TABLE.delete(db, id).await?;
    Ok(())
}
