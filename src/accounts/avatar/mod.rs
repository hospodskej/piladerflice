//! Profile pictures: processed by [`sanitize`], stored as the user's
//! `avatar` attachment (see [`storage`](crate::storage)) and shown only to
//! their owner.

pub mod sanitize;
pub mod webp_exif;

use std::path::Path;

use crate::db::Db;
use crate::storage::{self, Attachment, NewFile};

/// The stored picture of a user: its blob key and checksum.
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Avatar {
    pub blob_id: i64,
    pub key: String,
    pub checksum: Option<String>,
}

impl Avatar {
    /// "?v=…": the URL changes whenever the picture does, so browsers may
    /// keep it for a long time.
    pub fn version(&self) -> String {
        self.checksum.as_deref().unwrap_or_default().chars().take(12).collect()
    }
}

pub async fn find(db: &Db, user_id: i64) -> sqlx::Result<Option<Avatar>> {
    sqlx::query_as(
        "SELECT b.id AS blob_id, b.key, b.checksum
         FROM active_storage_attachments a JOIN active_storage_blobs b ON b.id = a.blob_id
         WHERE a.record_type = 'User' AND a.record_id = ? AND a.name = 'avatar'
         ORDER BY a.id DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
}

fn attachment(user_id: i64) -> Attachment<'static> {
    Attachment::new("User", user_id, "avatar")
}

/// Saves `webp` as the user's picture, replacing (and deleting) the old one.
pub async fn store(db: &Db, storage_root: &Path, user_id: i64, webp: &[u8]) -> anyhow::Result<()> {
    let file = NewFile {
        bytes: webp,
        filename: "avatar.webp",
        content_type: "image/webp",
        metadata: r#"{"identified":true,"width":256,"height":256,"analyzed":true}"#.into(),
    };
    storage::attach(db, storage_root, attachment(user_id), file).await?;
    Ok(())
}

/// Removes the user's picture (row and file).
pub async fn remove(db: &Db, storage_root: &Path, user_id: i64) -> anyhow::Result<()> {
    storage::detach(db, storage_root, attachment(user_id)).await
}

/// The stored picture's bytes.
pub async fn read(storage_root: &Path, avatar: &Avatar) -> Option<Vec<u8>> {
    storage::read(storage_root, &avatar.key).await
}
