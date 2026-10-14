//! The SQLite database: the same file the Rails app uses
//! (`storage/development.sqlite3` by default, or `DATABASE_URL`).
//!
//! Tests get a fresh in-memory database built from `db/schema.sql`.

use std::str::FromStr;

use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};

pub type Db = SqlitePool;

pub async fn connect(url: &str) -> Result<Db> {
    let options = SqliteConnectOptions::from_str(url)?
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5))
        .foreign_keys(true);
    Ok(SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?)
}

/// An empty database with the full schema, for tests.
pub async fn in_memory() -> Result<Db> {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
    // One connection: every connection to :memory: would be its own database.
    let db = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    sqlx::raw_sql(include_str!("../db/schema.sql")).execute(&db).await?;
    Ok(db)
}
