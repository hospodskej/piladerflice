//! Changing the tables. `db/schema.sql` is the database as the Rails app
//! left it (migration [`BASELINE`]); every change since is a file in
//! `db/migrations/`, named `<version>_<what>.sql` with the version a
//! timestamp (`20261101120000_add_note_to_orders.sql`), applied in order by
//! `piladerflice migrate`.
//!
//! Applied versions are written into `schema_migrations`, the table Rails
//! kept them in, so the history simply continues. The server refuses to
//! start while a migration is waiting: the code would expect columns that
//! aren't there yet.

use std::path::Path;

use anyhow::{Context, Result};
use chrono::{SubsecRound, Utc};

use super::Db;

/// The newest Rails migration: what `db/schema.sql` contains.
pub const BASELINE: &str = "20261014100000";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Migration {
    /// "20261101120000"
    pub version: String,
    /// "add_note_to_orders"
    pub name: String,
    pub sql: String,
}

/// The migrations in `dir`, oldest first. Other files (a README) are left out.
pub fn read_dir(dir: &Path) -> Result<Vec<Migration>> {
    let mut migrations = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(migrations),
        Err(error) => return Err(error).with_context(|| format!("reading {}", dir.display())),
    };
    for entry in entries {
        let path = entry?.path();
        let Some(file) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(stem) = file.strip_suffix(".sql") else {
            continue;
        };
        let (version, name) = stem
            .split_once('_')
            .filter(|(version, _)| version.len() == 14 && version.chars().all(|c| c.is_ascii_digit()))
            .with_context(|| format!("{file}: migrations are named <14-digit timestamp>_<name>.sql"))?;
        anyhow::ensure!(version > BASELINE, "{file}: must be newer than the schema ({BASELINE})");
        let sql = std::fs::read_to_string(&path).with_context(|| format!("reading {file}"))?;
        migrations.push(Migration {
            version: version.to_string(),
            name: name.to_string(),
            sql,
        });
    }
    migrations.sort_by(|a, b| a.version.cmp(&b.version));
    Ok(migrations)
}

/// The migrations not applied to this database yet.
pub async fn pending(db: &Db, migrations: &[Migration]) -> Result<Vec<Migration>> {
    let applied: Vec<String> = sqlx::query_scalar("SELECT version FROM schema_migrations")
        .fetch_all(db)
        .await
        .context("not the Pila Derflice database (no schema_migrations table)")?;
    anyhow::ensure!(
        applied.iter().any(|version| version.as_str() >= BASELINE),
        "the database is older than the schema this program starts from ({BASELINE}); \
         it needs the last Rails migrations first"
    );
    Ok(migrations
        .iter()
        .filter(|migration| !applied.contains(&migration.version))
        .cloned()
        .collect())
}

/// Applies the waiting migrations, each in its own transaction (a failing
/// one changes nothing and stops the rest). Returns what was applied.
pub async fn run(db: &Db, migrations: &[Migration]) -> Result<Vec<Migration>> {
    let waiting = pending(db, migrations).await?;
    for migration in &waiting {
        let mut tx = db.begin().await?;
        sqlx::raw_sql(&migration.sql)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("migration {}_{} failed", migration.version, migration.name))?;
        sqlx::query("INSERT INTO schema_migrations (version) VALUES (?)")
            .bind(&migration.version)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        tracing::info!(
            "migrated {}_{} at {}",
            migration.version,
            migration.name,
            Utc::now().trunc_subsecs(0)
        );
    }
    Ok(waiting)
}

/// At startup: refuses a database with migrations still waiting.
pub async fn check(db: &Db, migrations: &[Migration]) -> Result<()> {
    let waiting = pending(db, migrations).await?;
    if let Some(first) = waiting.first() {
        anyhow::bail!(
            "{} database migration(s) waiting, from {}_{}: back up the database, then run `piladerflice migrate`",
            waiting.len(),
            first.version,
            first.name
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir_with(files: &[(&str, &str)]) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "pila-migrations-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        for (name, sql) in files {
            std::fs::write(dir.join(name), sql).unwrap();
        }
        dir
    }

    async fn database_at_baseline() -> Db {
        let db = super::super::in_memory_schema_only().await.unwrap();
        sqlx::query("INSERT INTO schema_migrations (version) VALUES (?)")
            .bind(BASELINE)
            .execute(&db)
            .await
            .unwrap();
        db
    }

    #[tokio::test]
    async fn waiting_migrations_stop_the_server_until_applied() {
        let dir = dir_with(&[
            (
                "20261201000000_add_note.sql",
                "ALTER TABLE orders ADD COLUMN note varchar;",
            ),
            ("README.md", "not a migration"),
        ]);
        let migrations = read_dir(&dir).unwrap();
        assert_eq!(migrations.len(), 1);
        let db = database_at_baseline().await;
        let error = check(&db, &migrations).await.unwrap_err().to_string();
        assert!(error.contains("run `piladerflice migrate`"), "{error}");

        let applied = run(&db, &migrations).await.unwrap();
        assert_eq!(applied[0].name, "add_note");
        assert!(check(&db, &migrations).await.is_ok());
        sqlx::query("SELECT note FROM orders").fetch_all(&db).await.unwrap();
        // Running again changes nothing.
        assert!(run(&db, &migrations).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_failing_migration_changes_nothing() {
        let dir = dir_with(&[(
            "20261201000000_broken.sql",
            "ALTER TABLE orders ADD COLUMN note varchar; THIS IS NOT SQL;",
        )]);
        let migrations = read_dir(&dir).unwrap();
        let db = database_at_baseline().await;
        assert!(run(&db, &migrations).await.is_err());
        assert_eq!(pending(&db, &migrations).await.unwrap().len(), 1);
        assert!(sqlx::query("SELECT note FROM orders").fetch_all(&db).await.is_err());
    }

    #[tokio::test]
    async fn refuses_a_database_older_than_the_schema() {
        let db = super::super::in_memory_schema_only().await.unwrap();
        sqlx::query("INSERT INTO schema_migrations (version) VALUES ('20250101000000')")
            .execute(&db)
            .await
            .unwrap();
        let error = check(&db, &[]).await.unwrap_err().to_string();
        assert!(error.contains("older than the schema"), "{error}");
    }

    #[test]
    fn badly_named_files_are_reported() {
        let dir = dir_with(&[("add_note.sql", "")]);
        assert!(read_dir(&dir).is_err());
        let dir = dir_with(&[("20200101000000_old.sql", "")]);
        assert!(read_dir(&dir).is_err());
        assert!(read_dir(Path::new("/nonexistent/migrations")).unwrap().is_empty());
    }

    /// The project's own migrations are well-formed.
    #[test]
    fn the_projects_migrations_read() {
        read_dir(&Path::new(env!("CARGO_MANIFEST_DIR")).join("db/migrations")).unwrap();
    }
}
