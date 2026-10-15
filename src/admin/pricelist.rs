//! The price list (Ceník) on the contact page, edited as one table per
//! category. The public side is `models/pricelist.rs`.

use super::ordering;
use super::record::{Table, Values, integer, text};
use crate::db::Db;
use crate::i18n::Translations;
use crate::locale::Locale;
use crate::models::pricelist::FIREWOOD_CATEGORIES;
use crate::models::validation::{Errors, Validator};

/// The categories in page order, with the columns each table shows.
/// Kamenivo is a single price.
pub const CATEGORY_FIELDS: [(&str, &[&str]); 6] = [
    ("stavebni", &["item_name", "details", "price"]),
    ("palivove_volne", &["subcategory", "item_name", "price"]),
    ("palivove_skladane", &["subcategory", "item_name", "price"]),
    ("zbytky", &["item_name", "price"]),
    ("kamenivo", &["price"]),
    ("sluzby", &["item_name", "price"]),
];

/// Categories with exactly one item (no adding, no moving).
pub const SINGLE_ITEM_CATEGORIES: [&str; 1] = ["kamenivo"];

pub const TABLE: Table = Table {
    name: "pricelist_items",
    param: "pricelist_item",
    columns: &[
        text("category"),
        text("item_name"),
        text("item_name_de"),
        text("details"),
        text("details_de"),
        text("subcategory"),
        text("subcategory_de"),
        text("price"),
        text("price_de"),
        integer("position", Some("0")),
    ],
};

pub const PERMITTED: [&str; 9] = [
    "category",
    "item_name",
    "item_name_de",
    "details",
    "details_de",
    "subcategory",
    "subcategory_de",
    "price",
    "price_de",
];

pub fn fields_of(category: &str) -> &'static [&'static str] {
    CATEGORY_FIELDS
        .iter()
        .find(|(name, _)| *name == category)
        .map(|(_, fields)| *fields)
        .unwrap_or(&[])
}

/// Rails' order: a known category, a price, a name where the table has
/// one, the wood where it is firewood.
pub fn validate(translations: &Translations, values: &Values) -> Errors {
    let mut check = Validator::new(translations, Locale::Cs, "pricelist_item");
    let category = values.get("category").unwrap_or_default();
    if !CATEGORY_FIELDS.iter().any(|(name, _)| *name == category) {
        check.add("category", "inclusion");
    }
    if !values.is_present("price") {
        check.add("price", "blank");
    }
    if fields_of(category).contains(&"item_name") && !values.is_present("item_name") {
        check.add("item_name", "blank");
    }
    if FIREWOOD_CATEGORIES.contains(&category) && !values.is_present("subcategory") {
        check.add("subcategory", "blank");
    }
    check.finish()
}

/// The position for a new item: after the last one of its category.
pub async fn next_position(db: &Db, category: &str) -> sqlx::Result<i64> {
    let last: Option<i64> = sqlx::query_scalar("SELECT MAX(position) FROM pricelist_items WHERE category = ?")
        .bind(category)
        .fetch_one(db)
        .await?;
    Ok(last.unwrap_or(0) + 1)
}

/// Moves an item up (-1) or down (+1) within its category, numbering the
/// whole category afresh, so items sharing a position still move.
pub async fn move_item(db: &Db, id: i64, category: &str, direction: i64) -> sqlx::Result<()> {
    let mut ids: Vec<i64> =
        sqlx::query_scalar("SELECT id FROM pricelist_items WHERE category = ? ORDER BY position, id")
            .bind(category)
            .fetch_all(db)
            .await?;
    let Some((index, target)) = ordering::neighbour(&ids, id, direction) else {
        return Ok(());
    };
    ids.swap(index, target);
    let mut tx = db.begin().await?;
    for (position, id) in ids.iter().enumerate() {
        sqlx::query("UPDATE pricelist_items SET position = ? WHERE id = ?")
            .bind(position as i64)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await
}
