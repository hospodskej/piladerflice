//! A new database (`piladerflice setup`): the tables (`db/schema.sql`), the
//! starting content (`db/seeds.sql`: price list, catalogue, promos,
//! services...), the content's pictures (`db/seed_images.csv`, from
//! `assets/images`) and then the migrations. For a new development machine,
//! or a fresh start; the live site's database is moved over, not set up.

use std::path::Path;
use std::str::FromStr;

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

use super::{Db, migrations};
use crate::Config;
use crate::admin::images;
use crate::storage::{self, Attachment, NewFile};

/// Creates the database `config.database_url` names. Refuses to touch an
/// existing file.
pub async fn create(config: &Config) -> Result<Db> {
    let options = SqliteConnectOptions::from_str(&config.database_url)?;
    let file = options.get_filename().to_path_buf();
    anyhow::ensure!(
        !file.exists(),
        "{} already exists; setup only creates new databases",
        file.display()
    );
    if let Some(dir) = file.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)?;
    }
    let db = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            options
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .foreign_keys(true),
        )
        .await?;

    sqlx::raw_sql(include_str!("../../db/schema.sql")).execute(&db).await?;
    sqlx::query("INSERT INTO schema_migrations (version) VALUES (?)")
        .bind(migrations::BASELINE)
        .execute(&db)
        .await?;
    let seeds = std::fs::read_to_string(config.root.join("db/seeds.sql")).context("reading db/seeds.sql")?;
    sqlx::raw_sql(&seeds)
        .execute(&db)
        .await
        .context("loading db/seeds.sql")?;
    attach_seed_images(&db, config).await?;
    migrations::run(&db, &migrations::read_dir(&config.root.join("db/migrations"))?).await?;
    Ok(db)
}

/// `record_type,record_id,image` lines: the picture `assets/images/<image>`
/// becomes the record's `image`.
async fn attach_seed_images(db: &Db, config: &Config) -> Result<()> {
    let list = std::fs::read_to_string(config.root.join("db/seed_images.csv")).context("reading db/seed_images.csv")?;
    for line in list.lines().skip(1).filter(|line| !line.trim().is_empty()) {
        let mut fields = line.split(',');
        let (Some(record_type), Some(record_id), Some(image)) = (fields.next(), fields.next(), fields.next()) else {
            anyhow::bail!("db/seed_images.csv: bad line {line:?}");
        };
        let path = config.root.join("assets/images").join(image);
        let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        let picture = images::prepare(&bytes).map_err(|_| anyhow::anyhow!("{image} is not a picture"))?;
        let filename = Path::new(image)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(image);
        let file = NewFile {
            bytes: &picture.bytes,
            filename,
            content_type: picture.content_type,
            metadata: picture.metadata(),
        };
        let at = Attachment::new(record_type, record_id.parse()?, "image");
        storage::attach(db, &config.storage_root, at, file).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_new_database_has_the_starting_content_and_pictures() {
        let dir = std::env::temp_dir().join(format!("pila-setup-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut config = Config::from_source_tree(false);
        config.database_url = format!("sqlite:{}", dir.join("new.sqlite3").display());
        config.storage_root = dir.join("storage");

        let db = create(&config).await.unwrap();
        let count = |sql: &'static str| sqlx::query_scalar::<_, i64>(sql).fetch_one(&db);
        assert_eq!(count("SELECT COUNT(*) FROM catalog_products").await.unwrap(), 15);
        assert!(count("SELECT COUNT(*) FROM catalog_variants").await.unwrap() > 100);
        assert_eq!(count("SELECT COUNT(*) FROM promos").await.unwrap(), 3);
        assert_eq!(
            count("SELECT COUNT(*) FROM active_storage_attachments").await.unwrap(),
            18
        );
        assert_eq!(count("SELECT COUNT(*) FROM users").await.unwrap(), 0);
        // Ready to serve: nothing waiting.
        let known = migrations::read_dir(&config.root.join("db/migrations")).unwrap();
        migrations::check(&db, &known).await.unwrap();
        // The pictures are on disk where the site looks for them.
        let key: String = sqlx::query_scalar(
            "SELECT b.key FROM active_storage_blobs b JOIN active_storage_attachments a ON a.blob_id = b.id
             WHERE a.record_type = 'CatalogProduct' AND a.record_id = 1",
        )
        .fetch_one(&db)
        .await
        .unwrap();
        assert!(storage::read(&config.storage_root, &key).await.is_some());

        db.close().await;
        assert!(create(&config).await.is_err(), "never overwrites a database");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
