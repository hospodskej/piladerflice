//! Discount codes: a percentage or a fixed amount off the goods.

use crate::catalog::text_enum;
use crate::money::{round_div, Czk};
use serde::{Deserialize, Serialize};
use time::Date;

text_enum! {
    DiscountKind {
        Percent => "percent",
        Fixed => "fixed",
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscountCode {
    pub code: String,
    pub kind: DiscountKind,
    pub value: i64,
    pub active: bool,
    pub valid_until: Option<Date>,
    pub usage_limit: Option<i64>,
    pub times_used: i64,
    pub min_order: Option<Czk>,
}

/// Why a code can't be used right now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unusable {
    NotFound,
    Inactive,
    Expired,
    UsedUp,
    MinOrder,
}

impl Unusable {
    /// The translation key under `cart.discount.errors`.
    pub fn key(self) -> &'static str {
        match self {
            Unusable::NotFound => "not_found",
            Unusable::Inactive => "inactive",
            Unusable::Expired => "expired",
            Unusable::UsedUp => "used_up",
            Unusable::MinOrder => "min_order",
        }
    }
}

/// What a code may contain: A–Z, 0–9, - and _, at most 40 characters.
pub fn normalize_code(code: &str) -> Option<String> {
    let code = code.trim().to_uppercase();
    let valid = !code.is_empty()
        && code.len() <= 40
        && code.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-' || c == '_');
    valid.then_some(code)
}

impl DiscountCode {
    pub fn unusable_reason(&self, subtotal: Czk, today: Date) -> Option<Unusable> {
        if !self.active {
            Some(Unusable::Inactive)
        } else if self.valid_until.is_some_and(|d| d < today) {
            Some(Unusable::Expired)
        } else if self.usage_limit.is_some_and(|limit| self.times_used >= limit) {
            Some(Unusable::UsedUp)
        } else if self.min_order.is_some_and(|min| subtotal < min) {
            Some(Unusable::MinOrder)
        } else {
            None
        }
    }

    /// The discount on `subtotal`; a fixed amount never exceeds it.
    pub fn amount_for(&self, subtotal: Czk) -> Czk {
        match self.kind {
            DiscountKind::Percent => Czk(round_div(subtotal.get() * self.value, 100)),
            DiscountKind::Fixed => Czk(self.value.min(subtotal.get())),
        }
    }

    /// "10 %" or "200 Kč".
    pub fn label(&self) -> String {
        match self.kind {
            DiscountKind::Percent => format!("{} %", self.value),
            DiscountKind::Fixed => Czk(self.value).to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn code(kind: DiscountKind, value: i64) -> DiscountCode {
        DiscountCode {
            code: "JARO".into(),
            kind,
            value,
            active: true,
            valid_until: None,
            usage_limit: None,
            times_used: 0,
            min_order: None,
        }
    }

    #[test]
    fn amounts() {
        assert_eq!(code(DiscountKind::Percent, 10).amount_for(Czk(1745)), Czk(175));
        assert_eq!(code(DiscountKind::Fixed, 500).amount_for(Czk(300)), Czk(300));
        assert_eq!(code(DiscountKind::Fixed, 500).label(), "500 Kč");
        assert_eq!(code(DiscountKind::Percent, 15).label(), "15 %");
    }

    #[test]
    fn reasons() {
        let today = date!(2026 - 11 - 13);
        let mut c = code(DiscountKind::Percent, 10);
        assert_eq!(c.unusable_reason(Czk(100), today), None);
        c.valid_until = Some(date!(2026 - 11 - 12));
        assert_eq!(c.unusable_reason(Czk(100), today), Some(Unusable::Expired));
        c.valid_until = Some(today);
        assert_eq!(c.unusable_reason(Czk(100), today), None);
        c.usage_limit = Some(3);
        c.times_used = 3;
        assert_eq!(c.unusable_reason(Czk(100), today), Some(Unusable::UsedUp));
        c.times_used = 0;
        c.min_order = Some(Czk(1000));
        assert_eq!(c.unusable_reason(Czk(999), today), Some(Unusable::MinOrder));
        c.active = false;
        assert_eq!(c.unusable_reason(Czk(5000), today), Some(Unusable::Inactive));
    }

    #[test]
    fn normalizes_codes() {
        assert_eq!(normalize_code("  jaro-26 ").as_deref(), Some("JARO-26"));
        assert_eq!(normalize_code("sleva!"), None);
        assert_eq!(normalize_code(""), None);
        assert_eq!(normalize_code("ŽLUŤ"), None);
    }
}
