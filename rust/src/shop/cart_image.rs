//! The picture a cart line shows: the variant's own, or its product's.

use crate::db::Db;
use crate::models::catalog::{CatalogProduct, CatalogVariant};
use crate::models::upload::{self, Upload};

pub async fn for_variant(db: &Db, product: &CatalogProduct, variant: &CatalogVariant) -> sqlx::Result<Option<String>> {
    if let Some(own) = upload::images_of(db, "CatalogVariant").await?.get(&variant.id) {
        return Ok(Some(own.url()));
    }
    Ok(upload::images_of(db, "CatalogProduct")
        .await?
        .get(&product.id)
        .map(Upload::url))
}
