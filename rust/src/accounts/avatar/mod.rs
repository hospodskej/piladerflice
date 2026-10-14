//! Profile pictures: processed by [`sanitize`], stored the way Rails' Active
//! Storage stores them (a blob row, an attachment to the user, the file in
//! the storage folder) and shown only to their owner.

pub mod sanitize;
pub mod webp_exif;

use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{SubsecRound, Utc};
use md5::{Digest, Md5};
use rand::Rng;

use crate::db::Db;

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

/// Active Storage's file layout: storage/ab/cd/abcd…
pub fn file_for(root: &Path, key: &str) -> std::path::PathBuf {
    root.join(&key[0..2]).join(&key[2..4]).join(key)
}

/// A new blob key: 28 random lowercase letters and digits, like Rails'
/// `SecureRandom.base36(28)`.
fn new_key() -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..28)
        .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
        .collect()
}

/// Saves `webp` as the user's picture, replacing (and deleting) the old one.
pub async fn store(db: &Db, storage_root: &Path, user_id: i64, webp: &[u8]) -> anyhow::Result<()> {
    let key = new_key();
    let path = file_for(storage_root, &key);
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }
    tokio::fs::write(&path, webp).await?;

    let checksum = STANDARD.encode(Md5::digest(webp));
    let now = Utc::now().naive_utc().trunc_subsecs(6);
    let mut tx = db.begin().await?;
    let blob_id: i64 = sqlx::query_scalar(
        "INSERT INTO active_storage_blobs (key, filename, content_type, metadata, service_name, byte_size, checksum, created_at)
         VALUES (?, 'avatar.webp', 'image/webp', ?, 'local', ?, ?, ?) RETURNING id",
    )
    .bind(&key)
    .bind(r#"{"identified":true,"width":256,"height":256,"analyzed":true}"#)
    .bind(webp.len() as i64)
    .bind(&checksum)
    .bind(now)
    .fetch_one(&mut *tx)
    .await?;
    let old = old_blobs(&mut tx, user_id).await?;
    sqlx::query(
        "DELETE FROM active_storage_attachments WHERE record_type = 'User' AND record_id = ? AND name = 'avatar'",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO active_storage_attachments (name, record_type, record_id, blob_id, created_at)
         VALUES ('avatar', 'User', ?, ?, ?)",
    )
    .bind(user_id)
    .bind(blob_id)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    delete_blobs(&mut tx, &old).await?;
    tx.commit().await?;
    remove_files(storage_root, &old).await;
    Ok(())
}

/// Removes the user's picture (row and file).
pub async fn remove(db: &Db, storage_root: &Path, user_id: i64) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;
    let old = old_blobs(&mut tx, user_id).await?;
    sqlx::query(
        "DELETE FROM active_storage_attachments WHERE record_type = 'User' AND record_id = ? AND name = 'avatar'",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    delete_blobs(&mut tx, &old).await?;
    tx.commit().await?;
    remove_files(storage_root, &old).await;
    Ok(())
}

async fn old_blobs(tx: &mut sqlx::SqliteConnection, user_id: i64) -> sqlx::Result<Vec<(i64, String)>> {
    sqlx::query_as(
        "SELECT b.id, b.key FROM active_storage_attachments a JOIN active_storage_blobs b ON b.id = a.blob_id
         WHERE a.record_type = 'User' AND a.record_id = ? AND a.name = 'avatar'",
    )
    .bind(user_id)
    .fetch_all(tx)
    .await
}

async fn delete_blobs(tx: &mut sqlx::SqliteConnection, blobs: &[(i64, String)]) -> sqlx::Result<()> {
    for (id, _) in blobs {
        sqlx::query("DELETE FROM active_storage_blobs WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    Ok(())
}

async fn remove_files(storage_root: &Path, blobs: &[(i64, String)]) {
    for (_, key) in blobs {
        if key.len() >= 4 && key.chars().all(|c| c.is_ascii_alphanumeric()) {
            let _ = tokio::fs::remove_file(file_for(storage_root, key)).await;
        }
    }
}

/// The stored picture's bytes.
pub async fn read(storage_root: &Path, avatar: &Avatar) -> Option<Vec<u8>> {
    if avatar.key.len() < 4 || !avatar.key.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    tokio::fs::read(file_for(storage_root, &avatar.key)).await.ok()
}
