//! A Google review, copied into the site (`reviews`).

use chrono::{Datelike, NaiveDate};

use crate::db::Db;
use crate::locale::Locale;

/// A Google review, copied into the site.
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Review {
    pub id: i64,
    pub name: String,
    pub avatar: Option<String>,
    pub stars: i64,
    pub text: String,
    pub reviewed_on: NaiveDate,
}

const AVATAR_COLORS: [&str; 8] = [
    "#4285F4", "#EA4335", "#9C27B0", "#0F9D58", "#F4B400", "#FF7043", "#00ACC1", "#8E24AA",
];

impl Review {
    pub fn initial(&self) -> String {
        self.name
            .chars()
            .next()
            .map(|c| c.to_uppercase().collect())
            .unwrap_or_default()
    }

    pub fn avatar_color(&self) -> &'static str {
        AVATAR_COLORS[self.id.rem_euclid(AVATAR_COLORS.len() as i64) as usize]
    }

    /// "před 2 lety" / "vor 3 Monaten": how long ago, in whole years or months.
    pub fn time_ago(&self, locale: Locale, today: NaiveDate) -> String {
        let then = self.reviewed_on;
        let mut years = today.year() - then.year();
        if (today.month(), today.day()) < (then.month(), then.day()) {
            years -= 1;
        }
        let (count, cs_one, cs_many, de_one, de_many) = if years >= 1 {
            (years, "rokem", "lety", "Jahr", "Jahren")
        } else {
            let mut months = (today.year() - then.year()) * 12 + today.month() as i32 - then.month() as i32;
            if today.day() < then.day() {
                months -= 1;
            }
            (months.max(1), "měsícem", "měsíci", "Monat", "Monaten")
        };
        match (locale, count) {
            (Locale::De, 1) => format!("vor 1 {de_one}"),
            (Locale::De, n) => format!("vor {n} {de_many}"),
            (Locale::Cs, 1) => format!("před {cs_one}"),
            (Locale::Cs, n) => format!("před {n} {cs_many}"),
        }
    }
}

pub async fn reviews(db: &Db) -> sqlx::Result<Vec<Review>> {
    sqlx::query_as("SELECT id, name, avatar, stars, text, reviewed_on FROM reviews ORDER BY position, id")
        .fetch_all(db)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn review(on: &str) -> Review {
        Review {
            id: 3,
            name: "šimon".into(),
            avatar: None,
            stars: 5,
            text: String::new(),
            reviewed_on: NaiveDate::parse_from_str(on, "%Y-%m-%d").unwrap(),
        }
    }

    #[test]
    fn time_ago_in_years_or_months() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 14).unwrap();
        assert_eq!(review("2024-10-14").time_ago(Locale::Cs, today), "před 2 lety");
        assert_eq!(review("2025-10-15").time_ago(Locale::Cs, today), "před 11 měsíci");
        assert_eq!(review("2025-10-14").time_ago(Locale::De, today), "vor 1 Jahr");
        assert_eq!(review("2026-10-01").time_ago(Locale::Cs, today), "před měsícem");
        assert_eq!(review("2026-08-14").time_ago(Locale::De, today), "vor 2 Monaten");
    }

    #[test]
    fn avatar_initial_and_color() {
        assert_eq!(review("2026-01-01").initial(), "Š");
        assert_eq!(review("2026-01-01").avatar_color(), "#0F9D58");
    }
}
