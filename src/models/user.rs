//! A customer or admin account (`users`). The profile fields are the same
//! as the checkout details, so checkout can be filled in from them.

use chrono::{NaiveDateTime, SubsecRound, Utc};

use crate::db::Db;

pub const PHONE_PREFIXES: [&str; 2] = ["+420", "+43"];

/// The profile fields, in the order the profile form shows them.
pub const PROFILE_FIELDS: [&str; 16] = [
    "first_name",
    "last_name",
    "phone",
    "billing_street",
    "billing_city",
    "billing_zip",
    "billing_country",
    "company_purchase",
    "company_name",
    "company_ico",
    "company_dic",
    "delivery_address_different",
    "delivery_street",
    "delivery_city",
    "delivery_zip",
    "delivery_country",
];

#[derive(Clone, Debug, Default, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub email_address: String,
    pub password_digest: String,
    pub role: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub billing_street: Option<String>,
    pub billing_city: Option<String>,
    pub billing_zip: Option<String>,
    pub billing_country: Option<String>,
    pub company_purchase: bool,
    pub company_name: Option<String>,
    pub company_ico: Option<String>,
    pub company_dic: Option<String>,
    pub delivery_address_different: bool,
    pub delivery_street: Option<String>,
    pub delivery_city: Option<String>,
    pub delivery_zip: Option<String>,
    pub delivery_country: Option<String>,
    pub created_at: NaiveDateTime,
}

const COLUMNS: &str = "id, email_address, password_digest, role, first_name, last_name, phone, billing_street,
    billing_city, billing_zip, billing_country, company_purchase, company_name, company_ico, company_dic,
    delivery_address_different, delivery_street, delivery_city, delivery_zip, delivery_country, created_at";

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    /// "+43" when the phone starts with it, else "+420".
    pub fn phone_prefix(&self) -> &'static str {
        let phone = self.phone.as_deref().unwrap_or_default();
        PHONE_PREFIXES
            .into_iter()
            .find(|prefix| phone.starts_with(prefix))
            .unwrap_or(PHONE_PREFIXES[0])
    }

    /// The phone without its prefix.
    pub fn phone_number(&self) -> String {
        let phone = self.phone.as_deref().unwrap_or_default();
        phone
            .strip_prefix(self.phone_prefix())
            .unwrap_or(phone)
            .trim()
            .to_string()
    }

    /// A profile field as text ("true"/"false" for the two checkboxes), as
    /// checkout stores it.
    pub fn field(&self, name: &str) -> Option<String> {
        match name {
            "first_name" => self.first_name.clone(),
            "last_name" => self.last_name.clone(),
            "phone" => self.phone.clone(),
            "billing_street" => self.billing_street.clone(),
            "billing_city" => self.billing_city.clone(),
            "billing_zip" => self.billing_zip.clone(),
            "billing_country" => self.billing_country.clone(),
            "company_purchase" => Some(self.company_purchase.to_string()),
            "company_name" => self.company_name.clone(),
            "company_ico" => self.company_ico.clone(),
            "company_dic" => self.company_dic.clone(),
            "delivery_address_different" => Some(self.delivery_address_different.to_string()),
            "delivery_street" => self.delivery_street.clone(),
            "delivery_city" => self.delivery_city.clone(),
            "delivery_zip" => self.delivery_zip.clone(),
            "delivery_country" => self.delivery_country.clone(),
            _ => None,
        }
    }
}

/// E-mail addresses are compared trimmed and in lower case.
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

pub async fn find(db: &Db, id: i64) -> sqlx::Result<Option<User>> {
    sqlx::query_as(&format!("SELECT {COLUMNS} FROM users WHERE id = ?"))
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn find_by_email(db: &Db, email: &str) -> sqlx::Result<Option<User>> {
    sqlx::query_as(&format!("SELECT {COLUMNS} FROM users WHERE email_address = ?"))
        .bind(normalize_email(email))
        .fetch_optional(db)
        .await
}

/// A new customer account; returns its id.
pub async fn create_customer(db: &Db, email: &str, password_digest: &str) -> sqlx::Result<i64> {
    let now = Utc::now().naive_utc().trunc_subsecs(6);
    sqlx::query_scalar(
        "INSERT INTO users (email_address, password_digest, role, company_purchase, delivery_address_different,
           created_at, updated_at)
         VALUES (?, ?, 'customer', 0, 0, ?, ?) RETURNING id",
    )
    .bind(normalize_email(email))
    .bind(password_digest)
    .bind(now)
    .bind(now)
    .fetch_one(db)
    .await
}

/// Creates the admin account, or makes an existing account the admin with
/// this password (`piladerflice admin`). Returns its id.
pub async fn save_admin(db: &Db, email: &str, password_digest: &str) -> sqlx::Result<i64> {
    let now = Utc::now().naive_utc().trunc_subsecs(6);
    sqlx::query_scalar(
        "INSERT INTO users (email_address, password_digest, role, company_purchase, delivery_address_different,
           created_at, updated_at)
         VALUES (?, ?, 'admin', 0, 0, ?, ?)
         ON CONFLICT (email_address) DO UPDATE SET password_digest = excluded.password_digest, role = 'admin',
           updated_at = excluded.updated_at
         RETURNING id",
    )
    .bind(normalize_email(email))
    .bind(password_digest)
    .bind(now)
    .bind(now)
    .fetch_one(db)
    .await
}

/// Saves profile fields given as text (checkboxes as "true"/"false"); an
/// empty text is saved as empty. Unknown names are ignored.
pub async fn update_profile(db: &Db, user_id: i64, fields: &[(String, Option<String>)]) -> sqlx::Result<()> {
    let now = Utc::now().naive_utc().trunc_subsecs(6);
    for (name, value) in fields {
        if !PROFILE_FIELDS.contains(&name.as_str()) {
            continue;
        }
        // The column name comes from the fixed list above, never from the form.
        let sql = format!("UPDATE users SET {name} = ?, updated_at = ? WHERE id = ?");
        let query = sqlx::query(&sql);
        let query = if name == "company_purchase" || name == "delivery_address_different" {
            query.bind(value.as_deref() == Some("true"))
        } else {
            query.bind(value.clone())
        };
        query.bind(now).bind(user_id).execute(db).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_the_phone_into_prefix_and_number() {
        let user = User {
            phone: Some("+43 660 123".into()),
            ..User::default()
        };
        assert_eq!((user.phone_prefix(), user.phone_number().as_str()), ("+43", "660 123"));
        let user = User {
            phone: Some("602 000 000".into()),
            ..User::default()
        };
        assert_eq!(
            (user.phone_prefix(), user.phone_number().as_str()),
            ("+420", "602 000 000")
        );
        assert_eq!(normalize_email("  Jana@Example.CZ "), "jana@example.cz");
    }
}
