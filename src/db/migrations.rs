//! Changing the tables. Each change is a file in `db/migrations/`
//! (`<timestamp>_<what>.sql`), built into the program and applied in order
//! by `piladerflice migrate` (sqlx records them in `_sqlx_migrations`). The
//! server refuses to start while one is waiting: the code would expect
//! columns that aren't there yet.

use anyhow::{Context, Result};
use sqlx::migrate::Migrator;

use super::Db;

/// The migrations in `db/migrations/`, built into the program.
pub static MIGRATOR: Migrator = sqlx::migrate!("db/migrations");

/// Applies the waiting migrations; returns how many ran.
pub async fn run(db: &Db) -> Result<usize> {
    let before = waiting(db).await?.len();
    MIGRATOR.run(db).await.context("migrating the database")?;
    Ok(before)
}

/// At startup: refuses a database with migrations still waiting.
pub async fn check(db: &Db) -> Result<()> {
    let waiting = waiting(db).await?;
    if let Some(first) = waiting.first() {
        anyhow::bail!(
            "{} database migration(s) waiting, from {first}: back up the database, then run `piladerflice migrate`",
            waiting.len()
        );
    }
    Ok(())
}

/// The migrations (version_description) not applied to this database yet.
async fn waiting(db: &Db) -> Result<Vec<String>> {
    let has_table: bool =
        sqlx::query_scalar("SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'")
            .fetch_one(db)
            .await?;
    let applied: Vec<i64> = if has_table {
        sqlx::query_scalar("SELECT version FROM _sqlx_migrations WHERE success")
            .fetch_all(db)
            .await?
    } else {
        Vec::new()
    };
    Ok(MIGRATOR
        .iter()
        .filter(|migration| !applied.contains(&migration.version))
        .map(|migration| format!("{}_{}", migration.version, migration.description.replace(' ', "_")))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_new_database_waits_until_migrated() {
        let db = super::super::empty_in_memory().await.unwrap();
        let error = check(&db).await.unwrap_err().to_string();
        assert!(error.contains("run `piladerflice migrate`"), "{error}");
        assert!(error.contains("create_tables"), "{error}");

        assert_eq!(run(&db).await.unwrap(), MIGRATOR.iter().count());
        check(&db).await.unwrap();
        sqlx::query("SELECT id, key, image_id FROM catalog_products")
            .fetch_all(&db)
            .await
            .unwrap();
        // Running again changes nothing.
        assert_eq!(run(&db).await.unwrap(), 0);
    }
}
