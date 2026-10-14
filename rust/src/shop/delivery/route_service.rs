//! Driving distance from the sawmill to a delivery address, via
//! openrouteservice: the address is geocoded, then routed by car from
//! Derflice. Results are kept for 30 days per address. Any problem (no key,
//! address not found, service down) gives None, and checkout falls back to
//! "delivery price on request".

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const BASE_URL: &str = "https://api.openrouteservice.org";
/// Pila Derflice, Derflice 66 (lon, lat)
pub const ORIGIN: [f64; 2] = [16.12111, 48.80958];
const COUNTRIES: [&str; 2] = ["cz", "at"];
const TIMEOUT: Duration = Duration::from_secs(4);
const CACHE_TTL: Duration = Duration::from_secs(30 * 24 * 60 * 60);

/// Fetches a JSON document from a URL. Real HTTP in the app; tests swap in
/// a fake.
pub type Fetcher =
    Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = anyhow::Result<serde_json::Value>> + Send>> + Send + Sync>;

pub struct RouteService {
    api_key: Option<String>,
    fetch: Fetcher,
    cache: Mutex<HashMap<String, (i64, Instant)>>,
}

impl RouteService {
    pub fn new(api_key: Option<String>) -> Self {
        Self::with_fetcher(api_key, Arc::new(|url| Box::pin(http_get_json(url))))
    }

    pub fn with_fetcher(api_key: Option<String>, fetch: Fetcher) -> Self {
        Self {
            api_key: api_key.filter(|key| !key.is_empty()),
            fetch,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// One-way driving distance in whole kilometres, or None.
    pub async fn one_way_km(&self, street: &str, city: &str, zip: &str, country: &str) -> Option<i64> {
        let country = country.to_lowercase();
        let api_key = self.api_key.as_deref()?;
        if street.trim().is_empty() || city.trim().is_empty() || !COUNTRIES.contains(&country.as_str()) {
            return None;
        }
        let cache_key = [country.as_str(), zip, city, street]
            .iter()
            .map(|part| part.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase())
            .collect::<Vec<_>>()
            .join("|");
        if let Some((km, at)) = self.cache.lock().expect("route cache").get(&cache_key).copied()
            && at.elapsed() < CACHE_TTL
        {
            return Some(km);
        }

        match self.look_up(api_key, street, city, zip, &country).await {
            Ok(Some(km)) => {
                self.cache
                    .lock()
                    .expect("route cache")
                    .insert(cache_key, (km, Instant::now()));
                Some(km)
            }
            Ok(None) => None,
            Err(error) => {
                tracing::warn!("no delivery distance for {city} ({country}): {error:#}");
                None
            }
        }
    }

    async fn look_up(
        &self,
        key: &str,
        street: &str,
        city: &str,
        zip: &str,
        country: &str,
    ) -> anyhow::Result<Option<i64>> {
        // Built into a String before awaiting: the serializer isn't Send.
        let query = {
            let mut query = form_urlencoded::Serializer::new(String::new());
            query
                .append_pair("api_key", key)
                .append_pair("address", street)
                .append_pair("locality", city);
            if !zip.trim().is_empty() {
                query.append_pair("postalcode", zip);
            }
            query
                .append_pair("country", &country.to_uppercase())
                .append_pair("size", "1");
            query.finish()
        };
        let geocode = (self.fetch)(format!("{BASE_URL}/geocode/search/structured?{query}")).await?;
        let Some([lon, lat]) = geocode.pointer("/features/0/geometry/coordinates").and_then(as_point) else {
            return Ok(None);
        };

        let route_query = form_urlencoded::Serializer::new(String::new())
            .append_pair("api_key", key)
            .append_pair("start", &format!("{},{}", ORIGIN[0], ORIGIN[1]))
            .append_pair("end", &format!("{lon},{lat}"))
            .finish();
        let route = (self.fetch)(format!("{BASE_URL}/v2/directions/driving-car?{route_query}")).await?;
        let meters = route
            .pointer("/features/0/properties/summary/distance")
            .and_then(serde_json::Value::as_f64);
        Ok(meters.map(|meters| (meters / 1000.0).round() as i64))
    }
}

fn as_point(value: &serde_json::Value) -> Option<[f64; 2]> {
    let list = value.as_array()?;
    match list.as_slice() {
        [lon, lat] => Some([lon.as_f64()?, lat.as_f64()?]),
        _ => None,
    }
}

async fn http_get_json(url: String) -> anyhow::Result<serde_json::Value> {
    let response = reqwest::Client::builder()
        .timeout(TIMEOUT)
        .build()?
        .get(url)
        .header("Accept", "application/json, application/geo+json")
        .send()
        .await?
        .error_for_status()?;
    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fake(meters: f64) -> (RouteService, Arc<Mutex<Vec<String>>>) {
        let urls = Arc::new(Mutex::new(Vec::new()));
        let seen = urls.clone();
        let fetch: Fetcher = Arc::new(move |url: String| {
            seen.lock().unwrap().push(url.clone());
            Box::pin(async move {
                Ok(if url.contains("/geocode/") {
                    json!({ "features": [{ "geometry": { "coordinates": [16.6, 49.2] } }] })
                } else {
                    json!({ "features": [{ "properties": { "summary": { "distance": meters } } }] })
                })
            })
        });
        (RouteService::with_fetcher(Some("test-key".into()), fetch), urls)
    }

    #[tokio::test]
    async fn geocodes_routes_and_rounds() {
        let (service, urls) = fake(49_620.0);
        assert_eq!(service.one_way_km("Hlavní 1", "Brno", "60200", "cz").await, Some(50));
        let urls = urls.lock().unwrap();
        assert!(urls[0].contains("locality=Brno") && urls[0].contains("country=CZ"));
        assert!(urls[1].contains("start=16.12111%2C48.80958") && urls[1].contains("end=16.6%2C49.2"));
    }

    #[tokio::test]
    async fn caches_per_address() {
        let (service, urls) = fake(50_000.0);
        service.one_way_km("Hlavní 1", "Brno", "60200", "cz").await;
        service.one_way_km(" hlavní  1 ", "Brno", "60200", "CZ").await;
        assert_eq!(urls.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn nothing_without_key_address_or_supported_country() {
        let (service, _) = fake(1.0);
        assert_eq!(service.one_way_km("", "Brno", "1", "cz").await, None);
        assert_eq!(service.one_way_km("A 1", "Brno", "1", "sk").await, None);
        assert_eq!(RouteService::new(None).one_way_km("A 1", "Brno", "1", "cz").await, None);
    }
}
