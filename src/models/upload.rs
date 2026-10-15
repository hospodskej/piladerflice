//! Uploaded files: product, variant and promo photos and profile pictures.
//! Each is a row in `uploads` plus the file in the storage folder (see
//! [`crate::storage`]); a record points at its upload with `image_id`
//! (`avatar_id` for users).

use std::collections::HashMap;

use crate::db::Db;

/// What an upload belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Product,
    Variant,
    Promo,
    User,
}

impl Owner {
    pub(crate) fn table(self) -> &'static str {
        match self {
            Owner::Product => "catalog_products",
            Owner::Variant => "catalog_variants",
            Owner::Promo => "promos",
            Owner::User => "users",
        }
    }

    /// The column pointing at the upload.
    pub(crate) fn column(self) -> &'static str {
        match self {
            Owner::User => "avatar_id",
            _ => "image_id",
        }
    }

    /// "product", "variant", "promo" or "user" (in `db/seed_images.csv`).
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "product" => Some(Owner::Product),
            "variant" => Some(Owner::Variant),
            "promo" => Some(Owner::Promo),
            "user" => Some(Owner::User),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Upload {
    /// The record the upload belongs to.
    pub record_id: i64,
    pub key: String,
    pub filename: String,
    pub content_type: String,
}

impl Upload {
    pub fn url(&self) -> String {
        let filename: String = form_urlencoded::byte_serialize(self.filename.as_bytes()).collect();
        format!("/uploads/{}/{}", self.key, filename.replace('+', "%20"))
    }
}

fn select(owner: Owner, condition: &str) -> String {
    format!(
        "SELECT r.id AS record_id, u.key, u.filename, u.content_type
         FROM {table} r JOIN uploads u ON u.id = r.{column} {condition}",
        table = owner.table(),
        column = owner.column(),
    )
}

/// The upload of every record of one kind, by record id. The tables are
/// small, so loading them all is fine.
pub async fn images_of(db: &Db, owner: Owner) -> sqlx::Result<HashMap<i64, Upload>> {
    let uploads: Vec<Upload> = sqlx::query_as(&select(owner, "")).fetch_all(db).await?;
    Ok(uploads.into_iter().map(|upload| (upload.record_id, upload)).collect())
}

/// The upload of one record, if it has one.
pub async fn image_of(db: &Db, owner: Owner, record_id: i64) -> sqlx::Result<Option<Upload>> {
    sqlx::query_as(&select(owner, "WHERE r.id = ?"))
        .bind(record_id)
        .fetch_optional(db)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_escape_the_filename() {
        let upload = Upload {
            record_id: 1,
            key: "abcd".into(),
            filename: "dub štípaný.webp".into(),
            content_type: "image/webp".into(),
        };
        assert_eq!(upload.url(), "/uploads/abcd/dub%20%C5%A1t%C3%ADpan%C3%BD.webp");
    }
}
