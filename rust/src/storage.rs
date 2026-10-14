//! Files stored the way Rails' Active Storage stores them, so both apps see
//! the same pictures: a row in `active_storage_blobs` (key, name, type,
//! checksum), a row in `active_storage_attachments` tying it to a record
//! ("the `image` of CatalogProduct 3"), and the file itself in the storage
//! folder as `ab/cd/abcd…` (the key's first two pairs of characters as
//! folders).
//!
//! Used for profile pictures and for the images uploaded in the admin.

use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{SubsecRound, Utc};
use md5::{Digest, Md5};
use rand::Rng;

use crate::db::Db;

/// Which attachment of which record: `Attachment::new("Promo", 2, "image")`.
#[derive(Clone, Copy, Debug)]
pub struct Attachment<'a> {
    pub record_type: &'a str,
    pub record_id: i64,
    pub name: &'a str,
}

impl<'a> Attachment<'a> {
    pub fn new(record_type: &'a str, record_id: i64, name: &'a str) -> Self {
        Self {
            record_type,
            record_id,
            name,
        }
    }
}

/// A file to store, already checked.
pub struct NewFile<'a> {
    pub bytes: &'a [u8],
    pub filename: &'a str,
    pub content_type: &'a str,
    /// Active Storage's JSON, e.g. `{"identified":true,"width":800,...}`.
    pub metadata: String,
}

/// Blob keys are letters and digits (28 of them when made here or by
/// Rails); anything else is not a key and must never reach the file system.
pub fn valid_key(key: &str) -> bool {
    (4..=64).contains(&key.len()) && key.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Where a blob's file lives. Only for [`valid_key`]s.
pub fn file_for(root: &Path, key: &str) -> PathBuf {
    root.join(&key[0..2]).join(&key[2..4]).join(key)
}

/// A stored file's bytes.
pub async fn read(root: &Path, key: &str) -> Option<Vec<u8>> {
    if !valid_key(key) {
        return None;
    }
    tokio::fs::read(file_for(root, key)).await.ok()
}

/// Stores `file` as the attachment, replacing (and deleting) what was
/// attached before. Returns the new blob's key.
pub async fn attach(db: &Db, root: &Path, at: Attachment<'_>, file: NewFile<'_>) -> anyhow::Result<String> {
    let key = new_key();
    let path = file_for(root, &key);
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }
    tokio::fs::write(&path, file.bytes).await?;

    let checksum = STANDARD.encode(Md5::digest(file.bytes));
    let now = Utc::now().naive_utc().trunc_subsecs(6);
    let mut tx = db.begin().await?;
    let blob_id: i64 = sqlx::query_scalar(
        "INSERT INTO active_storage_blobs (key, filename, content_type, metadata, service_name, byte_size, checksum, created_at)
         VALUES (?, ?, ?, ?, 'local', ?, ?, ?) RETURNING id",
    )
    .bind(&key)
    .bind(file.filename)
    .bind(file.content_type)
    .bind(&file.metadata)
    .bind(file.bytes.len() as i64)
    .bind(&checksum)
    .bind(now)
    .fetch_one(&mut *tx)
    .await?;
    let old = remove_attachment(&mut tx, at).await?;
    sqlx::query(
        "INSERT INTO active_storage_attachments (name, record_type, record_id, blob_id, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(at.name)
    .bind(at.record_type)
    .bind(at.record_id)
    .bind(blob_id)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    remove_files(root, &old).await;
    Ok(key)
}

/// Removes the attachment, its blob and its file (Rails' `purge`).
pub async fn detach(db: &Db, root: &Path, at: Attachment<'_>) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;
    let old = remove_attachment(&mut tx, at).await?;
    tx.commit().await?;
    remove_files(root, &old).await;
    Ok(())
}

/// Deletes the attachment rows and their blobs; returns the blob keys,
/// whose files go once the transaction is through.
async fn remove_attachment(tx: &mut sqlx::SqliteConnection, at: Attachment<'_>) -> sqlx::Result<Vec<String>> {
    let blobs: Vec<(i64, String)> = sqlx::query_as(
        "SELECT b.id, b.key FROM active_storage_attachments a JOIN active_storage_blobs b ON b.id = a.blob_id
         WHERE a.record_type = ? AND a.record_id = ? AND a.name = ?",
    )
    .bind(at.record_type)
    .bind(at.record_id)
    .bind(at.name)
    .fetch_all(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM active_storage_attachments WHERE record_type = ? AND record_id = ? AND name = ?")
        .bind(at.record_type)
        .bind(at.record_id)
        .bind(at.name)
        .execute(&mut *tx)
        .await?;
    for (id, _) in &blobs {
        sqlx::query("DELETE FROM active_storage_blobs WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    Ok(blobs.into_iter().map(|(_, key)| key).collect())
}

async fn remove_files(root: &Path, keys: &[String]) {
    for key in keys.iter().filter(|key| valid_key(key)) {
        let _ = tokio::fs::remove_file(file_for(root, key)).await;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_letters_and_digits_are_keys() {
        assert!(valid_key("abcd1234"));
        assert!(!valid_key("../etc/passwd"));
        assert!(!valid_key("ab"));
        assert!(!valid_key("ab/cd"));
        assert!(!valid_key(""));
        let key = new_key();
        assert_eq!(key.len(), 28);
        assert!(valid_key(&key));
        assert_eq!(file_for(Path::new("/s"), "abcdef"), PathBuf::from("/s/ab/cd/abcdef"));
    }
}
