//! Variants (`catalog_variants`): what can actually be bought, each with
//! its own price — a length and amount of firewood, a size of beam...

use std::path::Path;

use super::messages::{blank, invalid, not_a_number, not_above, taken};
use super::record::{Table, Values, boolean, integer, parse_integer, text};
use crate::db::Db;
use crate::forms::Errors;
use crate::models::upload::Owner;
use crate::storage;

pub const VARIANT_GROUPS: [&str; 2] = ["kontejner", "bedny"];

pub const TABLE: Table = Table {
    name: "catalog_variants",
    columns: &[
        integer("catalog_product_id", "Produkt", None),
        text("key"),
        text("variant_group"),
        text("length_label"),
        text("variant_label"),
        text("variant_label_de"),
        integer("amount_value", "Množství", None),
        boolean("stacked", false),
        integer("price_czk", "Cena", None),
        boolean("in_stock", true),
        integer("position", "Pořadí", Some("0")),
        text("grade"),
        text("grade_de"),
        integer("width_mm", "Šířka", None),
        integer("height_mm", "Výška", None),
    ],
};

pub const FORM_FIELDS: [&str; 14] = [
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
];

/// Where the record's photo is stored.
pub fn image(id: i64) -> (Owner, i64) {
    (Owner::Variant, id)
}

/// The variant's checks, in this order: key given and not used by another
/// variant of the same product, a price above zero, a known group.
pub async fn validate(db: &Db, values: &Values, id: Option<i64>) -> sqlx::Result<Errors> {
    let mut errors = Errors::new();
    match values.get("key") {
        Some(key) if !key.trim().is_empty() => {
            let existing: Option<i64> = sqlx::query_scalar(
                "SELECT id FROM catalog_variants WHERE key = ? AND catalog_product_id = ? AND id IS NOT ?",
            )
            .bind(key)
            .bind(values.integer("catalog_product_id"))
            .bind(id)
            .fetch_optional(db)
            .await?;
            if existing.is_some() {
                errors.add("key", taken("Klíč"));
            }
        }
        _ => errors.add("key", blank("Klíč")),
    }
    // Whole crowns, as typed: "12 Kč" or "12.5" is an error, not 12.
    match values.get("price_czk").map(str::trim) {
        None | Some("") => errors.add("price_czk", not_a_number("Cena")),
        Some(price) => match parse_integer(price) {
            Some(price) if price > 0 => {}
            Some(_) => errors.add("price_czk", not_above("Cena", 0)),
            None => errors.add("price_czk", "Cena musí být celé číslo v korunách"),
        },
    }
    if let Some(group) = values.get("variant_group")
        && !VARIANT_GROUPS.contains(&group)
    {
        errors.add("variant_group", invalid("Způsob dodání"));
    }
    TABLE.check_numbers(values, &mut errors);
    Ok(errors)
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
