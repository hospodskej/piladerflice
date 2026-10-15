//! `piladerflice admin`: creates the admin account from `ADMIN_EMAIL` and
//! `ADMIN_PASSWORD` (in `.env` on a development machine), or gives an
//! existing account that e-mail the admin role and this password.

use anyhow::{Context, Result};

use crate::Config;
use crate::accounts::{password, registration};
use crate::db;
use crate::models::user;

pub async fn save(config: &Config) -> Result<()> {
    let email = std::env::var("ADMIN_EMAIL").context("set ADMIN_EMAIL (e.g. in .env)")?;
    let given = std::env::var("ADMIN_PASSWORD").context("set ADMIN_PASSWORD (e.g. in .env)")?;
    anyhow::ensure!(email.contains('@'), "ADMIN_EMAIL is not an e-mail address");
    anyhow::ensure!(
        given.chars().count() >= registration::MIN_PASSWORD,
        "ADMIN_PASSWORD must have at least {} characters",
        registration::MIN_PASSWORD
    );
    let db = db::connect(&config.database_url).await?;
    let id = user::save_admin(&db, &email, &password::hash(&given)?).await?;
    tracing::info!("admin account {} (id {id}) saved", user::normalize_email(&email));
    Ok(())
}
