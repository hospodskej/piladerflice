//! Prices. Everything is stored in Czech crowns; German (Austrian) visitors
//! see euros, converted at the day's ECB rate.
//!
//! - this file: formatting amounts ("2 500 Kč", "≈ 100,00 €")
//! - [`exchange_rate`]: the live rate, cached

mod exchange_rate;

pub use exchange_rate::{ExchangeRates, FALLBACK_RATE};

use std::sync::LazyLock;

use regex::Regex;

use crate::locale::Locale;

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
