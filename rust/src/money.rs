//! Prices. Everything is stored in Czech crowns; German (Austrian) visitors
//! see euros, converted at the live rate (port of Rails'
//! `ExchangeRateService`).
//!
//! The rate comes from frankfurter.dev and is kept for 6 hours. If the
//! service can't be reached, 25 CZK per EUR is used (and kept for 6 hours
//! too, so a dead service doesn't slow every page down).

use std::sync::LazyLock;
use std::time::{Duration, Instant};

use regex::Regex;
use tokio::sync::Mutex;

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

/// Formats crown amounts for one language: "2 500 Kč" or "≈ 100,00 €".
#[derive(Clone, Copy, Debug)]
pub struct Money {
    pub locale: Locale,
    pub rate: f64,
}

/// "2 500 Kč" or "2500 CZK" (the German texts sometimes spell it CZK).
/// Whitespace means ASCII whitespace, as in Ruby.
static AMOUNT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\d[\d \t\r\n\x0B\x0C]*)[ \t\r\n\x0B\x0C]*(?:Kč|CZK)").unwrap());

impl Money {
    pub fn display(&self, czk: i64) -> String {
        match self.locale {
            Locale::De => format!("≈ {}", format_eur(czk as f64 / self.rate)),
            Locale::Cs => format!("{} Kč", delimit(czk, " ")),
        }
    }

    /// Converts every crown amount in a free text to euros ("od 8500 Kč" ->
    /// "od ≈ 340,00 €"). Czech texts are returned unchanged.
    pub fn convert_text(&self, text: &str) -> String {
        if self.locale != Locale::De {
            return text.to_string();
        }
        AMOUNT_PATTERN
            .replace_all(text, |captures: &regex::Captures| {
                let digits: String = captures[1].chars().filter(|c| !c.is_ascii_whitespace()).collect();
                let czk: f64 = digits.parse().unwrap_or(0.0);
                format!("≈ {}", format_eur(czk / self.rate))
            })
            .into_owned()
    }
}

/// 1234567 -> "1 234 567"
pub fn delimit(number: i64, delimiter: &str) -> String {
    let digits = number.unsigned_abs().to_string();
    let mut out = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push_str(delimiter);
        }
        out.push(digit);
    }
    if number < 0 { format!("-{out}") } else { out }
}

/// 1234.5 -> "1.234,50 €"
pub fn format_eur(amount: f64) -> String {
    let cents = (amount * 100.0).round() as i64;
    let whole = delimit(cents / 100, ".");
    format!("{whole},{:02} €", (cents % 100).abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CZ: Money = Money {
        locale: Locale::Cs,
        rate: 25.0,
    };
    const AT: Money = Money {
        locale: Locale::De,
        rate: 25.0,
    };

    #[test]
    fn displays_crowns_and_euros() {
        assert_eq!(CZ.display(1500), "1 500 Kč");
        assert_eq!(CZ.display(21500), "21 500 Kč");
        assert_eq!(CZ.display(999), "999 Kč");
        assert_eq!(AT.display(1500), "≈ 60,00 €");
        assert_eq!(AT.display(4000), "≈ 160,00 €");
        assert_eq!(format_eur(1234.5), "1.234,50 €");
    }

    #[test]
    fn converts_amounts_inside_texts() {
        assert_eq!(AT.convert_text("2 500 Kč"), "≈ 100,00 €");
        assert_eq!(AT.convert_text("ab 8500 CZK / m³"), "ab ≈ 340,00 € / m³");
        assert_eq!(
            AT.convert_text("9 800 Kč / m<sup>3</sup>"),
            "≈ 392,00 € / m<sup>3</sup>"
        );
        assert_eq!(AT.convert_text("Po domluvě"), "Po domluvě");
        assert_eq!(CZ.convert_text("2 500 Kč"), "2 500 Kč");
    }
}
