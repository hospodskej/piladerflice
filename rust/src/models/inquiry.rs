//! A quote request from the contact page (`inquiries`): contact details and
//! the requested items (one row per item, the columns depend on the category).

use chrono::{NaiveDateTime, SubsecRound, Utc};

use crate::db::Db;

#[derive(Clone, Debug, Default)]
pub struct Inquiry {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub street: String,
    pub house_number: String,
    pub city: String,
    pub zip: String,
    pub notes: String,
    /// "palivove" or "stavebni"
    pub category: String,
    /// The rows, each a map of field -> value, as JSON.
    pub items_snapshot: String,
    pub locale: String,
    pub created_at: NaiveDateTime,
}

impl Inquiry {
    /// Name, e-mail and phone are required.
    pub fn is_valid(&self) -> bool {
        [&self.first_name, &self.last_name, &self.email, &self.phone]
            .iter()
            .all(|value| !value.trim().is_empty())
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name).trim().to_string()
    }

    /// "Hlavní 1, 602 00 Brno", leaving out what is empty.
    pub fn full_address(&self) -> String {
        let join = |parts: [&str; 2]| {
            parts
                .iter()
                .filter(|p| !p.trim().is_empty())
                .copied()
                .collect::<Vec<_>>()
                .join(" ")
        };
        [join([&self.street, &self.house_number]), join([&self.zip, &self.city])]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn items(&self) -> Vec<serde_json::Map<String, serde_json::Value>> {
        serde_json::from_str(&self.items_snapshot).unwrap_or_default()
    }

    pub async fn insert(mut self, db: &Db) -> sqlx::Result<Inquiry> {
        // Microseconds, like Rails stores them.
        let now = Utc::now().naive_utc().trunc_subsecs(6);
        self.id = sqlx::query_scalar(
            "INSERT INTO inquiries (first_name, last_name, email, phone, street, house_number, city, zip, notes,
               category, items_snapshot, locale, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(&self.first_name)
        .bind(&self.last_name)
        .bind(&self.email)
        .bind(&self.phone)
        .bind(&self.street)
        .bind(&self.house_number)
        .bind(&self.city)
        .bind(&self.zip)
        .bind(&self.notes)
        .bind(&self.category)
        .bind(&self.items_snapshot)
        .bind(&self.locale)
        .bind(now)
        .bind(now)
        .fetch_one(db)
        .await?;
        self.created_at = now;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_leaves_out_empty_parts() {
        let inquiry = Inquiry {
            street: "Hlavní".into(),
            house_number: "1".into(),
            city: "Brno".into(),
            ..Inquiry::default()
        };
        assert_eq!(inquiry.full_address(), "Hlavní 1, Brno");
        assert_eq!(Inquiry::default().full_address(), "");
    }
}
