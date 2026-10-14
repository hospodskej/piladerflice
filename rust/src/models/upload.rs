//! Images uploaded in the admin (product photos, promo pictures), stored
//! the way Rails' Active Storage stores them: a row in
//! `active_storage_blobs`, attached to a record in
//! `active_storage_attachments`, and the file on disk (see
//! `routes/uploads.rs`).

use std::collections::HashMap;

use crate::db::Db;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Upload {
    pub record_id: i64,
    pub key: String,
    pub filename: String,
    pub content_type: Option<String>,
}

impl Upload {
    pub fn url(&self) -> String {
        let filename: String = form_urlencoded::byte_serialize(self.filename.as_bytes()).collect();
        format!("/uploads/{}/{}", self.key, filename.replace('+', "%20"))
    }
}

/// The `image` of every record of one kind ("CatalogProduct", "Promo", ...),
/// by record id. The tables are small, so loading them all is fine.
pub async fn images_of(db: &Db, record_type: &str) -> sqlx::Result<HashMap<i64, Upload>> {
    let uploads: Vec<Upload> = sqlx::query_as(
        "SELECT a.record_id, b.key, b.filename, b.content_type
         FROM active_storage_attachments a JOIN active_storage_blobs b ON b.id = a.blob_id
         WHERE a.record_type = ? AND a.name = 'image'",
    )
    .bind(record_type)
    .fetch_all(db)
    .await?;
    Ok(uploads.into_iter().map(|upload| (upload.record_id, upload)).collect())
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
            content_type: None,
        };
        assert_eq!(upload.url(), "/uploads/abcd/dub%20%C5%A1t%C3%ADpan%C3%BD.webp");
    }
}
