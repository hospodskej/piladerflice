//! Crowns per euro. The rate comes from frankfurter.dev and is kept for 6
//! hours. If the service can't be reached, 25 CZK per EUR is used (and kept
//! for 6 hours too, so a dead service doesn't slow every page down).

use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use super::Money;
use crate::locale::Locale;

const RATE_ENDPOINT: &str = "https://api.frankfurter.dev/v2/rate/EUR/CZK";
const CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);
pub const FALLBACK_RATE: f64 = 25.0;

pub struct ExchangeRates {
    cached: Mutex<Option<(f64, Instant)>>,
    /// Tests use a fixed rate and never call the service.
    fixed: Option<f64>,
}

impl ExchangeRates {
    pub fn live() -> Self {
        Self {
            cached: Mutex::new(None),
            fixed: None,
        }
    }

    pub fn fixed(rate: f64) -> Self {
        Self {
            cached: Mutex::new(None),
            fixed: Some(rate),
        }
    }

    /// Crowns per euro. The lock is held while fetching, so many visitors
    /// arriving at once cause one request, not one each.
    pub async fn czk_per_eur(&self) -> f64 {
        if let Some(rate) = self.fixed {
            return rate;
        }
        let mut cached = self.cached.lock().await;
        if let Some((rate, at)) = *cached
            && at.elapsed() < CACHE_TTL
        {
            return rate;
        }
        let rate = fetch_live_rate().await.unwrap_or_else(|error| {
            tracing::warn!("live exchange rate fetch failed ({error}); using fallback {FALLBACK_RATE}");
            FALLBACK_RATE
        });
        *cached = Some((rate, Instant::now()));
        rate
    }

    /// A formatter for the current language, with the rate already looked up.
    pub async fn money(&self, locale: Locale) -> Money {
        let rate = match locale {
            Locale::De => self.czk_per_eur().await,
            Locale::Cs => FALLBACK_RATE, // not used for crowns
        };
        Money { locale, rate }
    }
}

async fn fetch_live_rate() -> anyhow::Result<f64> {
    #[derive(serde::Deserialize)]
    struct Answer {
        rate: Option<f64>,
    }
    let answer: Answer = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()?
        .get(RATE_ENDPOINT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    answer
        .rate
        .filter(|rate| *rate > 0.0)
        .ok_or_else(|| anyhow::anyhow!("response missing \"rate\" field"))
}
