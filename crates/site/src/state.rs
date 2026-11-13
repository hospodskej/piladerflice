//! What every request can reach on the server.

use domain::money::EurRate;
use sqlx::SqlitePool;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: SqlitePool,
    /// Crowns per euro, refreshed in the background.
    pub eur: Arc<RwLock<EurRate>>,
    /// Counts repeated attempts (sign-in, forms) per address or account.
    pub limiter: refractory::Limiter,
    /// Road distances (openrouteservice); `None` without ORS_API_KEY, and
    /// then delivery is "calculated individually".
    pub routes: Option<Arc<crate::routing::Routes>>,
    /// Signs the e-mailed password reset links (from SECRET_KEY_BASE).
    pub reset_tokens: thymus::ResetTokens,
    /// Where uploaded files are kept (STORAGE_DIR, default "storage").
    pub storage: std::path::PathBuf,
}

impl AppState {
    pub fn new(db: SqlitePool) -> AppState {
        let routes = crate::routing::client(db.clone()).map(Arc::new);
        AppState {
            db,
            eur: Arc::new(RwLock::new(EurRate::FALLBACK)),
            limiter: refractory::Limiter::new(),
            routes,
            reset_tokens: thymus::ResetTokens::new(&secret_key()),
            storage: std::env::var("STORAGE_DIR").unwrap_or_else(|_| "storage".into()).into(),
        }
    }

    pub fn eur_rate(&self) -> EurRate {
        self.eur.read().map(|r| *r).unwrap_or(EurRate::FALLBACK)
    }
}

/// The key that signs links (SECRET_KEY_BASE, at least 32 bytes). Required
/// in production; development uses a fixed key so links survive restarts.
fn secret_key() -> Vec<u8> {
    match std::env::var("SECRET_KEY_BASE") {
        Ok(key) if key.len() >= thymus::ResetTokens::MIN_KEY_LEN => key.into_bytes(),
        _ if std::env::var("APP_ENV").is_ok_and(|e| e == "production") => {
            panic!("SECRET_KEY_BASE must be set (at least 32 characters) in production")
        }
        _ => {
            tracing::warn!("SECRET_KEY_BASE isn't set; using the development key");
            b"piladerflice-development-key-not-for-production".to_vec()
        }
    }
}
