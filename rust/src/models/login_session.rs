//! A login (`sessions`): one row per signed-in browser, with when, from
//! where and which browser. Logging out deletes the row, which ends that
//! login; deleting all rows of a user would sign them out everywhere.

use chrono::{SubsecRound, Utc};

use super::user::{self, User};
use crate::db::Db;

/// Starts a login; returns the row id the login cookie will carry.
pub async fn start(db: &Db, user_id: i64, ip_address: Option<&str>, user_agent: Option<&str>) -> sqlx::Result<i64> {
    let now = Utc::now().naive_utc().trunc_subsecs(6);
    sqlx::query_scalar(
        "INSERT INTO sessions (user_id, ip_address, user_agent, created_at, updated_at) VALUES (?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(user_id)
    .bind(ip_address)
    .bind(user_agent)
    .bind(now)
    .bind(now)
    .fetch_one(db)
    .await
}

/// The user of a login, if it still exists.
pub async fn user_of(db: &Db, session_id: i64) -> sqlx::Result<Option<User>> {
    let user_id: Option<i64> = sqlx::query_scalar("SELECT user_id FROM sessions WHERE id = ?")
        .bind(session_id)
        .fetch_optional(db)
        .await?;
    match user_id {
        Some(id) => user::find(db, id).await,
        None => Ok(None),
    }
}

pub async fn end(db: &Db, session_id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(session_id)
        .execute(db)
        .await?;
    Ok(())
}
