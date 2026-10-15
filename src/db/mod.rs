//! The SQLite database (`storage/development.sqlite3` by default, or
//! `DATABASE_URL`). Its tables come from the Rails app (`db/schema.sql`);
//! changes since are `db/migrations/*.sql`.
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

/// An empty database with the full schema, for tests.
/// It is up to date: the schema plus this project's migrations.
pub async fn in_memory() -> Result<Db> {
    let db = in_memory_schema_only().await?;
    sqlx::query("INSERT INTO schema_migrations (version) VALUES (?)")
        .bind(migrations::BASELINE)
        .execute(&db)
        .await?;
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("db/migrations");
    migrations::run(&db, &migrations::read_dir(&dir)?).await?;
    Ok(db)
}

/// The tables as the Rails app left them, nothing recorded as applied.
pub async fn in_memory_schema_only() -> Result<Db> {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
    // One connection: every connection to :memory: would be its own database.
    let db = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    sqlx::raw_sql(include_str!("../../db/schema.sql")).execute(&db).await?;
    Ok(db)
}
