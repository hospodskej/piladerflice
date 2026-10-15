//! A new database (`piladerflice setup`): the tables (the migrations), the
//! starting content (`db/seeds.sql`: price list, catalogue, promos,
//! services...) and the content's pictures (`db/seed_images.csv`, from
//! `assets/images`). For a new development machine, or the first start of
//! the live site.

use std::path::Path;
use std::str::FromStr;

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

use super::{Db, migrations};
use crate::Config;
use crate::admin::images;
use crate::models::upload::Owner;
use crate::storage::{self, NewFile};

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

    migrations::run(&db).await?;
    let seeds = std::fs::read_to_string(config.root.join("db/seeds.sql")).context("reading db/seeds.sql")?;
    sqlx::raw_sql(&seeds)
        .execute(&db)
        .await
        .context("loading db/seeds.sql")?;
    attach_seed_images(&db, config).await?;
    Ok(db)
}

/// `owner,record_id,image` lines (`product,1,eshop/smrk.webp`): the picture
/// `assets/images/<image>` becomes the record's photo.
async fn attach_seed_images(db: &Db, config: &Config) -> Result<()> {
    let list = std::fs::read_to_string(config.root.join("db/seed_images.csv")).context("reading db/seed_images.csv")?;
    for line in list.lines().skip(1).filter(|line| !line.trim().is_empty()) {
        let mut fields = line.split(',');
        let (Some(owner), Some(record_id), Some(image)) = (fields.next(), fields.next(), fields.next()) else {
            anyhow::bail!("db/seed_images.csv: bad line {line:?}");
        };
        let owner = Owner::from_name(owner).with_context(|| format!("db/seed_images.csv: unknown owner {owner:?}"))?;
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
        };
        storage::attach(db, &config.storage_root, owner, record_id.parse()?, file).await?;
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
        assert_eq!(count("SELECT COUNT(*) FROM uploads").await.unwrap(), 18);
        assert_eq!(count("SELECT COUNT(*) FROM users").await.unwrap(), 0);
        // Ready to serve: nothing waiting.
        migrations::check(&db).await.unwrap();
        // The pictures are on disk where the site looks for them.
        let key: String = sqlx::query_scalar(
            "SELECT u.key FROM uploads u JOIN catalog_products p ON p.image_id = u.id WHERE p.id = 1",
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
