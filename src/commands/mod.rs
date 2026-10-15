//! What the program does besides serving the site, as
//! `piladerflice <command>`:
//!
//! - `setup`: a new database with the starting content ([`crate::db::setup`])
//! - `migrate`: apply waiting database migrations ([`crate::db::migrations`])
//! - `admin`: create the admin account, or reset its password
//! - `backup <file>`: copy the database while the site runs

mod admin;

use std::path::Path;

use anyhow::Result;

use crate::Config;
use crate::db;

/// Runs the command in `args`; Ok(false) when there is none (serve the site).
pub async fn run(args: &[String], config: &Config) -> Result<bool> {
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [] => return Ok(false),
        ["setup"] => {
            db::setup::create(config).await?;
            tracing::info!("database created: {}", config.database_url);
        }
        ["migrate"] => {
            let db = db::connect(&config.database_url).await?;
            let applied = db::migrations::run(&db).await?;
            tracing::info!("{applied} migration(s) applied, the database is up to date");
        }
        ["admin"] => admin::save(config).await?,
        ["backup", target] => {
            let db = db::connect(&config.database_url).await?;
            db::backup::copy_to(&db, Path::new(target)).await?;
            tracing::info!("database copied to {target}");
        }
        _ => anyhow::bail!("usage: piladerflice [setup | migrate | admin | backup <file>]"),
    }
    Ok(true)
}
