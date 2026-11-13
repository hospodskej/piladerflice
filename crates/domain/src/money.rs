//! Prices: whole crowns, shown as "1 740 Kč" or, on the German site, as
//! "≈ 69,60 €" at the current exchange rate.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Mul, Neg, Sub};

/// An amount in whole Czech crowns. Can be negative (a discount line).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Czk(pub i64);

impl Czk {
    pub const ZERO: Czk = Czk(0);

    pub fn get(self) -> i64 {
        self.0
    }

    pub fn is_positive(self) -> bool {
        self.0 > 0
    }

    /// The VAT contained in this amount (VAT-inclusive) at `rate` per cent,
    /// rounded half away from zero, like the old site.
    pub fn vat_in(self, rate: u8) -> Czk {
        Czk(round_div(self.0 * i64::from(rate), 100 + i64::from(rate)))
    }

    /// This amount without VAT (the amount being VAT-exclusive) plus `rate`
    /// per cent VAT.
    pub fn plus_vat(self, rate: u8) -> Czk {
        Czk(round_div(self.0 * (100 + i64::from(rate)), 100))
    }

    /// "1 740" (thin groups of three, separated by a space).
    pub fn grouped(self) -> String {
        group_thousands(self.0, ' ')
    }
}

/// Integer division rounded half away from zero (Ruby's `Float#round`).
pub fn round_div(numerator: i64, denominator: i64) -> i64 {
    debug_assert!(denominator > 0);
    numerator.signum() * ((numerator.abs() * 2 + denominator) / (denominator * 2))
}

fn group_thousands(value: i64, separator: char) -> String {
    let digits = value.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    if value < 0 {
        out.push('-');
    }
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(separator);
        }
        out.push(c);
    }
    out
}

impl fmt::Display for Czk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} Kč", self.grouped())
    }
}

impl Add for Czk {
    type Output = Czk;
    fn add(self, rhs: Czk) -> Czk {
        Czk(self.0 + rhs.0)
    }
}

impl AddAssign for Czk {
    fn add_assign(&mut self, rhs: Czk) {
        self.0 += rhs.0;
    }
}

impl Sub for Czk {
    type Output = Czk;
    fn sub(self, rhs: Czk) -> Czk {
        Czk(self.0 - rhs.0)
    }
}

impl Neg for Czk {
    type Output = Czk;
    fn neg(self) -> Czk {
        Czk(-self.0)
    }
}

impl Mul<i64> for Czk {
    type Output = Czk;
    fn mul(self, rhs: i64) -> Czk {
        Czk(self.0 * rhs)
    }
}

impl Sum for Czk {
    fn sum<I: Iterator<Item = Czk>>(iter: I) -> Czk {
        Czk(iter.map(|c| c.0).sum())
    }
}

/// Crowns per euro. Never zero or negative (see [`EurRate::new`]).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct EurRate(f64);

impl EurRate {
    /// Used when the day's rate can't be fetched.
    pub const FALLBACK: EurRate = EurRate(25.0);

    /// `None` unless the rate is a finite number above zero.
    pub fn new(czk_per_eur: f64) -> Option<EurRate> {
        (czk_per_eur.is_finite() && czk_per_eur > 0.0).then_some(EurRate(czk_per_eur))
    }

    pub fn czk_per_eur(self) -> f64 {
        self.0
    }

    /// "≈ 69,60 €".
    pub fn approx_eur(self, amount: Czk) -> String {
        format!("≈ {}", format_eur(amount.0 as f64 / self.0))
    }

    /// Replaces every "1 740 Kč" (or "CZK") in an admin-written price text with
    /// its euro value: "od 1 740 Kč / m³" → "od ≈ 69,60 € / m³".
    pub fn convert_price_text(self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let bytes = text.as_bytes();
        let mut i = 0;
        while i < text.len() {
            let c = text[i..].chars().next().unwrap_or(' ');
            if c.is_ascii_digit() {
                // digits and spaces, then optional spaces, then Kč/CZK
                let start = i;
                let mut j = i;
                while j < text.len() && (bytes[j].is_ascii_digit() || bytes[j] == b' ') {
                    j += 1;
                }
                let mut digits_end = j;
                while digits_end > start && bytes[digits_end - 1] == b' ' {
                    digits_end -= 1;
                }
                let rest = &text[j..];
                let unit = ["Kč", "CZK"].into_iter().find(|u| rest.starts_with(u));
                if let Some(unit) = unit {
                    let amount: i64 = text[start..digits_end]
                        .chars()
                        .filter(char::is_ascii_digit)
                        .collect::<String>()
                        .parse()
                        .unwrap_or(0);
                    out.push_str(&self.approx_eur(Czk(amount)));
                    i = j + unit.len();
                    continue;
                }
                out.push_str(&text[start..j]);
                i = j;
                continue;
            }
            out.push(c);
            i += c.len_utf8();
        }
        out
    }
}

/// "1.234,50 €".
fn format_eur(amount: f64) -> String {
    let cents = (amount * 100.0).round() as i64;
    let euros = group_thousands(cents / 100, '.');
    format!("{},{:02} €", euros, (cents % 100).abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_crowns() {
        assert_eq!(Czk(1740).to_string(), "1 740 Kč");
        assert_eq!(Czk(1_234_567).to_string(), "1 234 567 Kč");
        assert_eq!(Czk(5).to_string(), "5 Kč");
        assert_eq!(Czk(-1200).to_string(), "-1 200 Kč");
    }

    #[test]
    fn vat_rounds_like_ruby() {
        // 1000 * 21 / 121 = 173.55 -> 174
        assert_eq!(Czk(1000).vat_in(21), Czk(174));
        assert_eq!(Czk(121).vat_in(21), Czk(21));
        assert_eq!(Czk(0).vat_in(21), Czk(0));
        assert_eq!(Czk(-1000).vat_in(21), Czk(-174));
        // 1000 * 1.21 = 1210; 999 * 1.21 = 1208.79 -> 1209
        assert_eq!(Czk(999).plus_vat(21), Czk(1209));
        assert_eq!(Czk(50).plus_vat(13), Czk(57)); // 56.5 -> 57
    }

    #[test]
    fn round_div_half_away_from_zero() {
        assert_eq!(round_div(5, 10), 1);
        assert_eq!(round_div(4, 10), 0);
        assert_eq!(round_div(-5, 10), -1);
        assert_eq!(round_div(15, 10), 2);
        assert_eq!(round_div(1, 3), 0);
        assert_eq!(round_div(2, 3), 1);
    }

    #[test]
    fn converts_price_texts() {
        let rate = EurRate::new(25.0).unwrap();
        assert_eq!(rate.convert_price_text("1 740 Kč"), "≈ 69,60 €");
        assert_eq!(rate.convert_price_text("od 9 800 Kč / m<sup>3</sup>"), "od ≈ 392,00 € / m<sup>3</sup>");
        assert_eq!(rate.convert_price_text("40 CZK / km"), "≈ 1,60 € / km");
        assert_eq!(rate.convert_price_text("4m, 5m"), "4m, 5m");
        assert_eq!(rate.convert_price_text("---"), "---");
        assert_eq!(rate.approx_eur(Czk(2_500_000)), "≈ 100.000,00 €");
    }

    #[test]
    fn rejects_bad_rates() {
        assert!(EurRate::new(0.0).is_none());
        assert!(EurRate::new(f64::NAN).is_none());
        assert!(EurRate::new(-3.0).is_none());
    }
}
