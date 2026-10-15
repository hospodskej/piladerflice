//! Storing uploaded files: the file goes into the storage folder
//! (`storage/uploads/<key>`), a row into `uploads`, and the record that owns
//! it points at that row. Replacing or removing a file deletes the old one.

use std::path::{Path, PathBuf};

use chrono::Utc;
use rand::Rng;

use crate::db::Db;
use crate::models::upload::Owner;

/// A file to store, already checked.
pub struct NewFile<'a> {
    pub bytes: &'a [u8],
    pub filename: &'a str,
    pub content_type: &'a str,
}

/// Keys are letters and digits (28 of them); anything else is not a key and
/// must never reach the file system.
pub fn valid_key(key: &str) -> bool {
    (4..=64).contains(&key.len()) && key.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Where a file lives. Only for [`valid_key`]s.
pub fn file_for(root: &Path, key: &str) -> PathBuf {
    root.join(key)
}

/// A stored file's bytes.
pub async fn read(root: &Path, key: &str) -> Option<Vec<u8>> {
    if !valid_key(key) {
        return None;
    }
    tokio::fs::read(file_for(root, key)).await.ok()
}

/// Stores `file` as the record's upload, replacing (and deleting) the one
/// it had. Returns the new key.
pub async fn attach(db: &Db, root: &Path, owner: Owner, record_id: i64, file: NewFile<'_>) -> anyhow::Result<String> {
    let key = new_key();
    tokio::fs::create_dir_all(root).await?;
    tokio::fs::write(file_for(root, &key), file.bytes).await?;

    let mut tx = db.begin().await?;
    let upload_id: i64 = sqlx::query_scalar(
        "INSERT INTO uploads (key, filename, content_type, byte_size, created_at) VALUES (?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(&key)
    .bind(file.filename)
    .bind(file.content_type)
    .bind(file.bytes.len() as i64)
    .bind(Utc::now().naive_utc())
    .fetch_one(&mut *tx)
    .await?;
    let old = point_at(&mut tx, owner, record_id, Some(upload_id)).await?;
    tx.commit().await?;
    remove_file(root, old).await;
    Ok(key)
}

/// Removes the record's upload: the row and the file.
pub async fn detach(db: &Db, root: &Path, owner: Owner, record_id: i64) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;
    let old = point_at(&mut tx, owner, record_id, None).await?;
    tx.commit().await?;
    remove_file(root, old).await;
    Ok(())
}

/// Points the record at `upload_id` (or nothing) and deletes the upload it
/// pointed at before; returns that upload's key, whose file goes once the
/// transaction is through.
async fn point_at(
    tx: &mut sqlx::SqliteConnection,
    owner: Owner,
    record_id: i64,
    upload_id: Option<i64>,
) -> sqlx::Result<Option<String>> {
    let (table, column) = (owner.table(), owner.column());
    let old: Option<i64> = sqlx::query_scalar(&format!("SELECT {column} FROM {table} WHERE id = ?"))
        .bind(record_id)
        .fetch_optional(&mut *tx)
        .await?
        .flatten();
    sqlx::query(&format!("UPDATE {table} SET {column} = ? WHERE id = ?"))
        .bind(upload_id)
        .bind(record_id)
        .execute(&mut *tx)
        .await?;
    match old {
        Some(old) => {
            sqlx::query_scalar("DELETE FROM uploads WHERE id = ? RETURNING key")
                .bind(old)
                .fetch_optional(&mut *tx)
                .await
        }
        None => Ok(None),
    }
}

async fn remove_file(root: &Path, key: Option<String>) {
    if let Some(key) = key.filter(|key| valid_key(key)) {
        let _ = tokio::fs::remove_file(file_for(root, &key)).await;
    }
}

/// A new key: 28 random lowercase letters and digits.
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
        assert_eq!(file_for(Path::new("/s"), "abcdef"), PathBuf::from("/s/abcdef"));
    }

    #[tokio::test]
    async fn replacing_and_removing_deletes_the_old_file() {
        let db = crate::db::in_memory().await.unwrap();
        sqlx::query("INSERT INTO promos (id, created_at, updated_at) VALUES (1, '2026-01-01', '2026-01-01')")
            .execute(&db)
            .await
            .unwrap();
        let root = std::env::temp_dir().join(format!("pila-storage-{}", std::process::id()));
        let file = |bytes: &'static [u8]| NewFile {
            bytes,
            filename: "a.png",
            content_type: "image/png",
        };

        let first = attach(&db, &root, Owner::Promo, 1, file(b"one")).await.unwrap();
        let second = attach(&db, &root, Owner::Promo, 1, file(b"two")).await.unwrap();
        assert!(read(&root, &first).await.is_none(), "the replaced file is gone");
        assert_eq!(read(&root, &second).await.unwrap(), b"two");
        let image = crate::models::upload::image_of(&db, Owner::Promo, 1)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(image.key, second);

        detach(&db, &root, Owner::Promo, 1).await.unwrap();
        assert!(read(&root, &second).await.is_none());
        assert!(
            crate::models::upload::image_of(&db, Owner::Promo, 1)
                .await
                .unwrap()
                .is_none()
        );
        let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM uploads")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(rows, 0);
        let _ = std::fs::remove_dir_all(root);
    }
}
