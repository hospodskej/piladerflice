//! The SQLite database (`storage/development.sqlite3` by default, or
//! `DATABASE_URL`). Its tables are made by the migrations in
//! `db/migrations/`.
//!
//! - this file: connecting; tests get a fresh in-memory database
//! - [`migrations`]: changing the tables, and refusing a database that
//!   isn't up to date
//! - [`setup`]: a new database with the starting content
//! - [`backup`]: a copy while the site runs

pub mod backup;
pub mod migrations;
pub mod setup;

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

/// A database with every table and nothing in it, for tests.
pub async fn in_memory() -> Result<Db> {
    let db = empty_in_memory().await?;
    migrations::run(&db).await?;
    Ok(db)
}

/// A database without even the tables.
pub async fn empty_in_memory() -> Result<Db> {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
    // One connection: every connection to :memory: would be its own database.
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?)
}
