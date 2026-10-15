//! A consistent copy of the database, taken while the site keeps running
//! (`piladerflice backup <file>`, used by the nightly backup).

use anyhow::Result;

use super::Db;

/// A consistent copy of the database in a new file, taken while the site
/// keeps running (SQLite's `VACUUM INTO`). The file must not exist yet.
pub async fn copy_to(db: &Db, target: &std::path::Path) -> Result<()> {
    anyhow::ensure!(!target.exists(), "{} already exists", target.display());
    sqlx::query("VACUUM INTO ?")
        .bind(target.to_string_lossy().into_owned())
        .execute(db)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::connect;
    use super::*;

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
        copy_to(&source, &target).await.unwrap();
        let copy = connect(&format!("sqlite:{}", target.display())).await.unwrap();
        let x: i64 = sqlx::query_scalar("SELECT x FROM t").fetch_one(&copy).await.unwrap();
        assert_eq!(x, 42);
        assert!(copy_to(&source, &target).await.is_err(), "never overwrites");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
