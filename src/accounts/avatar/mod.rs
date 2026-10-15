//! Profile pictures: processed by [`sanitize`], stored as the user's
//! upload (see [`storage`](crate::storage)) and shown only to their owner.

pub mod sanitize;
pub mod webp_exif;

use std::path::Path;

use crate::db::Db;
use crate::models::upload::Owner;
use crate::storage::{self, NewFile};

/// The stored picture of a user.
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Avatar {
    pub key: String,
}

impl Avatar {
    /// "?v=…": the URL changes whenever the picture does (a new picture has a
    /// new key), so browsers may keep it for a long time.
    pub fn version(&self) -> String {
        self.key.chars().take(12).collect()
    }
}

pub async fn find(db: &Db, user_id: i64) -> sqlx::Result<Option<Avatar>> {
    sqlx::query_as("SELECT u.key FROM users JOIN uploads u ON u.id = users.avatar_id WHERE users.id = ?")
        .bind(user_id)
        .fetch_optional(db)
        .await
}

/// Saves `webp` as the user's picture, replacing (and deleting) the old one.
pub async fn store(db: &Db, storage_root: &Path, user_id: i64, webp: &[u8]) -> anyhow::Result<()> {
    let file = NewFile {
        bytes: webp,
        filename: "avatar.webp",
        content_type: "image/webp",
    };
    storage::attach(db, storage_root, Owner::User, user_id, file).await?;
    Ok(())
}

/// Removes the user's picture (row and file).
pub async fn remove(db: &Db, storage_root: &Path, user_id: i64) -> anyhow::Result<()> {
    storage::detach(db, storage_root, Owner::User, user_id).await
}

/// The stored picture's bytes.
pub async fn read(storage_root: &Path, avatar: &Avatar) -> Option<Vec<u8>> {
    storage::read(storage_root, &avatar.key).await
}
