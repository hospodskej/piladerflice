//! What every request can reach on the server.

use sqlx::SqlitePool;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: SqlitePool,
}
