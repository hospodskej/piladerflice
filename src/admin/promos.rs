//! The promos ("Doporučujeme") that take turns on the home page, one per
//! calendar week. Nothing is required: an empty promo just shows little.

use std::path::Path;

use super::record::{Table, text};
use crate::db::Db;
use crate::models::upload::Owner;
use crate::storage;

pub const TABLE: Table = Table {
    name: "promos",
    columns: &[
        text("title"),
        text("feature_1"),
        text("feature_2"),
        text("feature_3"),
        text("price"),
        text("link"),
        text("title_de"),
        text("feature_1_de"),
        text("feature_2_de"),
        text("feature_3_de"),
        text("price_de"),
    ],
};

pub const FORM_FIELDS: [&str; 11] = [
    "title",
    "feature_1",
    "feature_2",
    "feature_3",
    "price",
    "link",
    "title_de",
    "feature_1_de",
    "feature_2_de",
    "feature_3_de",
    "price_de",
];

/// Where the record's photo is stored.
pub fn image(id: i64) -> (Owner, i64) {
    (Owner::Promo, id)
}

pub async fn delete(db: &Db, storage_root: &Path, id: i64) -> anyhow::Result<()> {
    storage::detach(db, storage_root, Owner::Promo, id).await?;
    TABLE.delete(db, id).await?;
    Ok(())
}
