//! Road distances from the sawmill, through openrouteservice (the chemotaxis
//! crate), cached in SQLite for 30 days per address or place.

use crate::state::AppState;
use chemotaxis::{Address, Cache, Client, LatLon};
use domain::delivery::DeliveryEstimate;
use domain::vat::Country;
use sqlx::SqlitePool;
use std::time::Duration;

/// Pila Derflice, Derflice 66.
pub const SAWMILL: LatLon = LatLon { lat: 48.80958, lon: 16.12111 };

/// openrouteservice answers kept in the `route_cache` table.
#[derive(Clone, Debug)]
pub struct SqliteCache(SqlitePool);

impl Cache for SqliteCache {
    async fn get(&self, key: &str) -> Option<String> {
        sqlx::query_scalar!(
            "SELECT result FROM route_cache WHERE query = ? AND expires_at > strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
            key
        )
        .fetch_optional(&self.0)
        .await
        .ok()
        .flatten()
        .flatten()
    }

    async fn put(&self, key: &str, value: String, ttl: Duration) {
        let expires = (time::OffsetDateTime::now_utc() + ttl).format(&time::format_description::well_known::Rfc3339).unwrap_or_default();
        if let Err(e) = sqlx::query!("INSERT OR REPLACE INTO route_cache (query, result, expires_at) VALUES (?, ?, ?)", key, value, expires)
            .execute(&self.0)
            .await
        {
            tracing::warn!("route not cached: {e}");
        }
    }
}

pub type Routes = Client<SqliteCache>;

/// The client, when ORS_API_KEY is set.
pub fn client(db: SqlitePool) -> Option<Routes> {
    let key = std::env::var("ORS_API_KEY").ok().filter(|k| !k.trim().is_empty())?;
    match Client::builder(key, SAWMILL).countries(["cz", "at"]).cache(SqliteCache(db)).build() {
        Ok(client) => Some(client),
        Err(e) => {
            tracing::error!("openrouteservice client not set up: {e}");
            None
        }
    }
}

/// The approximate delivery to a town or PSČ, for "Kam vozíme?".
pub async fn estimate(state: &AppState, query: &str, country: Country) -> Option<DeliveryEstimate> {
    let routes = state.routes.as_ref()?;
    match routes.estimate(query, country.as_str()).await {
        Ok(found) => Some(DeliveryEstimate {
            query: query.to_owned(),
            place: found.place,
            zip: found.zip,
            country,
            one_way_km: i64::from(found.km),
        }),
        Err(e) => {
            if !e.is_not_found() {
                tracing::warn!("delivery estimate failed: {e}");
            }
            None
        }
    }
}

/// The one-way road distance to a full address, for checkout.
pub async fn one_way_km(state: &AppState, street: &str, city: &str, zip: &str, country: Country) -> Option<i64> {
    let routes = state.routes.as_ref()?;
    match routes.one_way_km(&Address { street, city, zip, country: country.as_str() }).await {
        Ok(km) => Some(i64::from(km)),
        Err(e) => {
            if !e.is_not_found() {
                tracing::warn!("delivery distance failed: {e}");
            }
            None
        }
    }
}
