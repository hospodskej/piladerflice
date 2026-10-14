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

/// The newest Rails migration this program knows the tables of (the
/// schema in `db/schema.sql`). When Rails gets a new migration, update
/// both; a test fails until then.
pub const SCHEMA_VERSION: &str = "20261014100000";

/// Refuses a database that is older than this program expects (columns
/// would be missing); warns about a newer one (Rails was migrated, and
/// this program may need updating too).
pub async fn check_schema(db: &Db) -> Result<()> {
    let version: Option<String> = sqlx::query_scalar("SELECT MAX(version) FROM schema_migrations")
        .fetch_one(db)
        .await
        .map_err(|error| anyhow::anyhow!("not the Pila Derflice database (no schema_migrations table): {error}"))?;
    let version = version.unwrap_or_default();
    match version.as_str().cmp(SCHEMA_VERSION) {
        std::cmp::Ordering::Less => anyhow::bail!(
            "the database is at migration {version}, but this program needs {SCHEMA_VERSION}: \
             run the Rails migrations first (bin/rails db:migrate)"
        ),
        std::cmp::Ordering::Greater => {
            tracing::warn!("the database is at migration {version}, newer than {SCHEMA_VERSION} this program knows")
        }
        std::cmp::Ordering::Equal => {}
    }
    Ok(())
}

/// A consistent copy of the database in a new file, taken while the site
/// keeps running (SQLite's `VACUUM INTO`). The file must not exist yet.
pub async fn backup(db: &Db, target: &std::path::Path) -> Result<()> {
    anyhow::ensure!(!target.exists(), "{} already exists", target.display());
    sqlx::query("VACUUM INTO ?")
        .bind(target.to_string_lossy().into_owned())
        .execute(db)
        .await?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A new Rails migration means the schema (and maybe the code) here
    /// needs updating: re-export `db/schema.sql`, then raise SCHEMA_VERSION.
    #[test]
    fn knows_the_newest_rails_migration() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../db/migrate");
        let newest = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| entry.ok()?.file_name().to_str()?.split('_').next().map(str::to_string))
            .max()
            .unwrap();
        assert_eq!(newest, SCHEMA_VERSION);
    }

    #[tokio::test]
    async fn backs_up_into_a_new_file() {
        let dir = std::env::temp_dir().join(format!("pila-backup-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let source = connect(&format!("sqlite:{}?mode=rwc", dir.join("live.sqlite3").display()))
            .await
            .unwrap();
        sqlx::raw_sql("CREATE TABLE t (x); INSERT INTO t VALUES (42);")
            .execute(&source)
            .await
            .unwrap();
        let target = dir.join("copy.sqlite3");
        backup(&source, &target).await.unwrap();
        let copy = connect(&format!("sqlite:{}", target.display())).await.unwrap();
        let x: i64 = sqlx::query_scalar("SELECT x FROM t").fetch_one(&copy).await.unwrap();
        assert_eq!(x, 42);
        assert!(backup(&source, &target).await.is_err(), "never overwrites");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn refuses_an_older_database() {
        let db = in_memory().await.unwrap();
        sqlx::query("INSERT INTO schema_migrations (version) VALUES ('20250101000000')")
            .execute(&db)
            .await
            .unwrap();
        let error = check_schema(&db).await.unwrap_err().to_string();
        assert!(error.contains("run the Rails migrations first"), "{error}");
        sqlx::query("INSERT INTO schema_migrations (version) VALUES (?)")
            .bind(SCHEMA_VERSION)
            .execute(&db)
            .await
            .unwrap();
        assert!(check_schema(&db).await.is_ok());
    }
}
