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
}

impl AppState {
    pub fn new(db: SqlitePool) -> AppState {
        let routes = crate::routing::client(db.clone()).map(Arc::new);
        AppState { db, eur: Arc::new(RwLock::new(EurRate::FALLBACK)), limiter: refractory::Limiter::new(), routes }
    }

    pub fn eur_rate(&self) -> EurRate {
        self.eur.read().map(|r| *r).unwrap_or(EurRate::FALLBACK)
    }
}
