//! An order (`orders`): who, where, how, what (a snapshot of the cart lines)
//! and the totals, all incl. VAT.

use chrono::{NaiveDateTime, SubsecRound, Utc};

use crate::db::Db;
use crate::i18n::Translations;
use crate::locale::Locale;
use crate::shop::cart_line::CartLine;

pub const SHIPPING_METHODS: [&str; 2] = ["address", "pickup"];
pub const PAYMENT_METHODS: [&str; 2] = ["cash", "bank_transfer"];

#[derive(Clone, Debug, Default, sqlx::FromRow)]
pub struct Order {
    pub id: i64,
    pub user_id: Option<i64>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub newsletter_opt_in: bool,
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
    pub shipping_method: Option<String>,
    pub payment_method: Option<String>,
    /// The cart lines as JSON, in the Rails app's format.
    pub items_snapshot: String,
    pub subtotal_czk: i64,
    /// Round trip, when the delivery price was calculated.
    pub delivery_distance_km: Option<i64>,
    pub delivery_czk: Option<i64>,
    pub vat_czk: i64,
    pub total_czk: i64,
    pub locale: String,
    pub created_at: NaiveDateTime,
}

impl Order {
    pub fn items(&self) -> Vec<CartLine> {
        #[derive(serde::Deserialize)]
        struct Snapshot {
            /// Text in orders from the Rails app ("106"), maybe a number.
            #[serde(default)]
            catalog_variant_id: serde_json::Value,
            product_key: String,
            image: Option<String>,
            unit_price_czk: i64,
            #[serde(default)]
            specs: Vec<crate::shop::cart_line::Spec>,
            quantity: i64,
        }
        let snapshots: Vec<Snapshot> = serde_json::from_str(&self.items_snapshot).unwrap_or_default();
        snapshots
            .into_iter()
            .map(|item| {
                let variant_id = match &item.catalog_variant_id {
                    serde_json::Value::String(text) => text.parse().unwrap_or_default(),
                    other => other.as_i64().unwrap_or_default(),
                };
                CartLine {
                    id: variant_id.to_string(),
                    catalog_variant_id: variant_id,
                    product_key: item.product_key,
                    image: item.image,
                    unit_price_czk: item.unit_price_czk,
                    specs: item.specs,
                    quantity: item.quantity,
                }
            })
            .collect()
    }

    pub fn full_name(&self) -> String {
        format!(
            "{} {}",
            self.first_name.as_deref().unwrap_or_default(),
            self.last_name.as_deref().unwrap_or_default()
        )
        .trim()
        .to_string()
    }

    /// Whether a delivery price was worked out at checkout (otherwise it is
    /// agreed individually).
    pub fn delivery_quoted(&self) -> bool {
        self.shipping_method.as_deref() == Some("address") && self.delivery_czk.is_some()
    }

    /// What is wrong with the order, as messages for the customer.
    pub fn errors(&self, translations: &Translations, locale: Locale) -> Vec<String> {
        let blank = |value: &Option<String>| value.as_deref().is_none_or(|text| text.trim().is_empty());
        let message = |field: &str| {
            let key = format!("activerecord.errors.models.order.attributes.{field}.blank");
            match translations.lookup(locale, &key).and_then(|value| value.as_str()) {
                Some(text) => text.to_string(),
                None => translations.text(locale, "checkout.submit_error", &[]),
            }
        };
        let mut errors = Vec::new();
        let required = [
            ("first_name", &self.first_name),
            ("last_name", &self.last_name),
            ("email", &self.email),
            ("phone", &self.phone),
            ("billing_street", &self.billing_street),
            ("billing_city", &self.billing_city),
            ("billing_zip", &self.billing_zip),
            ("billing_country", &self.billing_country),
        ];
        for (field, value) in required {
            if blank(value) {
                errors.push(message(field));
            }
        }
        if !SHIPPING_METHODS.contains(&self.shipping_method.as_deref().unwrap_or_default())
            || !PAYMENT_METHODS.contains(&self.payment_method.as_deref().unwrap_or_default())
            || self.items_snapshot.trim().is_empty()
        {
            errors.push(translations.text(locale, "checkout.submit_error", &[]));
        }
        if self.company_purchase && blank(&self.company_name) {
            errors.push(message("company_name"));
        }
        if self.delivery_address_different {
            for (field, value) in [
                ("delivery_street", &self.delivery_street),
                ("delivery_city", &self.delivery_city),
                ("delivery_zip", &self.delivery_zip),
                ("delivery_country", &self.delivery_country),
            ] {
                if blank(value) {
                    errors.push(message(field));
                }
            }
        }
        errors.dedup();
        errors
    }

    /// Saves a new order; returns it with its id and time.
    pub async fn insert(mut self, db: &Db) -> sqlx::Result<Order> {
        // Microseconds, like Rails stores them.
        let now = Utc::now().naive_utc().trunc_subsecs(6);
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO orders (user_id, first_name, last_name, email, phone, newsletter_opt_in,
               billing_street, billing_city, billing_zip, billing_country,
               company_purchase, company_name, company_ico, company_dic,
               delivery_address_different, delivery_street, delivery_city, delivery_zip, delivery_country,
               shipping_method, payment_method, items_snapshot, subtotal_czk, delivery_distance_km, delivery_czk,
               vat_czk, total_czk, locale, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             RETURNING id",
        )
        .bind(self.user_id)
        .bind(&self.first_name)
        .bind(&self.last_name)
        .bind(&self.email)
        .bind(&self.phone)
        .bind(self.newsletter_opt_in)
        .bind(&self.billing_street)
        .bind(&self.billing_city)
        .bind(&self.billing_zip)
        .bind(&self.billing_country)
        .bind(self.company_purchase)
        .bind(&self.company_name)
        .bind(&self.company_ico)
        .bind(&self.company_dic)
        .bind(self.delivery_address_different)
        .bind(&self.delivery_street)
        .bind(&self.delivery_city)
        .bind(&self.delivery_zip)
        .bind(&self.delivery_country)
        .bind(&self.shipping_method)
        .bind(&self.payment_method)
        .bind(&self.items_snapshot)
        .bind(self.subtotal_czk)
        .bind(self.delivery_distance_km)
        .bind(self.delivery_czk)
        .bind(self.vat_czk)
        .bind(self.total_czk)
        .bind(&self.locale)
        .bind(now)
        .bind(now)
        .fetch_one(db)
        .await?;
        self.id = id;
        self.created_at = now;
        Ok(self)
    }

    pub async fn find(db: &Db, id: i64) -> sqlx::Result<Option<Order>> {
        sqlx::query_as(
            "SELECT id, user_id, first_name, last_name, email, phone, newsletter_opt_in, billing_street, billing_city,
               billing_zip, billing_country, company_purchase, company_name, company_ico, company_dic,
               delivery_address_different, delivery_street, delivery_city, delivery_zip, delivery_country,
               shipping_method, payment_method, items_snapshot, subtotal_czk, delivery_distance_km, delivery_czk,
               vat_czk, total_czk, locale, created_at
             FROM orders WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(db)
        .await
    }
}

/// "Česká republika" / "Tschechien" for a country code.
pub fn country_name(code: Option<&str>, locale: Locale) -> String {
    match (locale, code.unwrap_or_default()) {
        (Locale::Cs, "cz") => "Česká republika".into(),
        (Locale::Cs, "at") => "Rakousko".into(),
        (Locale::De, "cz") => "Tschechien".into(),
        (Locale::De, "at") => "Österreich".into(),
        (_, other) => other.to_string(),
    }
}
