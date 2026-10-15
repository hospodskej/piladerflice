//! Editing a table row from a Rails-style form (`catalog_product[title]`).
//!
//! Each editable table is described once as a [`Table`]: its columns and
//! what kind each is. A record's values are kept as the text a form shows
//! ([`Values`]), so a form that didn't save shows exactly what was typed;
//! they are converted to numbers and true/false only when written, the way
//! Rails casts them.

use std::collections::BTreeMap;

use chrono::{SubsecRound, Utc};
use serde::Serialize;
use sqlx::Row;

use crate::db::Db;
use crate::forms::FormData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Text,
    Integer,
    Boolean,
}

#[derive(Clone, Copy, Debug)]
pub struct Column {
    pub name: &'static str,
    pub kind: Kind,
    /// The database default a new record starts with.
    pub default: Option<&'static str>,
}

pub const fn text(name: &'static str) -> Column {
    Column {
        name,
        kind: Kind::Text,
        default: None,
    }
}

pub const fn integer(name: &'static str, default: Option<&'static str>) -> Column {
    Column {
        name,
        kind: Kind::Integer,
        default,
    }
}

pub const fn boolean(name: &'static str, default: bool) -> Column {
    Column {
        name,
        kind: Kind::Boolean,
        default: Some(if default { "1" } else { "0" }),
    }
}

/// An editable table: `catalog_products`, whose form fields are named
/// `catalog_product[...]`.
pub struct Table {
    pub name: &'static str,
    pub param: &'static str,
    pub columns: &'static [Column],
}

/// A record's values as form text; None is NULL.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Values(BTreeMap<String, Option<String>>);

impl Values {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0.get(name).and_then(|value| value.as_deref())
    }

    pub fn set(&mut self, name: &str, value: Option<String>) {
        self.0.insert(name.to_string(), value);
    }

    /// A checkbox's state, read like Rails reads booleans.
    pub fn is_true(&self, name: &str) -> bool {
        self.get(name).is_some_and(truthy)
    }

    /// A number column as Rails casts it ("12 kg" is 12, "abc" is NULL).
    pub fn integer(&self, name: &str) -> Option<i64> {
        self.get(name).and_then(cast_integer)
    }

    /// Present and not just spaces (Rails' `present?`).
    pub fn is_present(&self, name: &str) -> bool {
        self.get(name).is_some_and(|value| !value.trim().is_empty())
    }
}

impl Table {
    fn column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|column| column.name == name)
    }

    /// A new record: the database defaults, everything else empty.
    pub fn new_record(&self) -> Values {
        let mut values = Values::default();
        for column in self.columns {
            values.set(column.name, column.default.map(str::to_string));
        }
        values
    }

    /// The record with this id, or None.
    pub async fn find(&self, db: &Db, id: i64) -> sqlx::Result<Option<Values>> {
        let columns: Vec<String> = self
            .columns
            .iter()
            .map(|column| format!("CAST({0} AS TEXT) AS {0}", column.name))
            .collect();
        // Table and column names are constants of this program, never input.
        let sql = format!("SELECT {} FROM {} WHERE id = ?", columns.join(", "), self.name);
        let Some(row) = sqlx::query(&sql).bind(id).fetch_optional(db).await? else {
            return Ok(None);
        };
        let mut values = Values::default();
        for column in self.columns {
            values.set(column.name, row.try_get::<Option<String>, _>(column.name)?);
        }
        Ok(Some(values))
    }

    /// Takes the submitted `param[name]` fields that may be changed
    /// (Rails' `params.permit`); fields not sent stay as they are.
    pub fn assign(&self, values: &mut Values, form: &FormData, permitted: &[&str]) {
        for name in permitted {
            if self.column(name).is_none() {
                continue;
            }
            if let Some(value) = form.get(&format!("{}[{name}]", self.param)) {
                values.set(name, Some(value.to_string()));
            }
        }
    }

    pub async fn insert(&self, db: &Db, values: &Values) -> sqlx::Result<i64> {
        let names: Vec<&str> = self.columns.iter().map(|column| column.name).collect();
        let sql = format!(
            "INSERT INTO {} ({}, created_at, updated_at) VALUES ({}, ?, ?) RETURNING id",
            self.name,
            names.join(", "),
            vec!["?"; names.len()].join(", ")
        );
        let now = Utc::now().naive_utc().trunc_subsecs(6);
        let mut query = sqlx::query_scalar(&sql);
        for column in self.columns {
            query = bind(query, column, values);
        }
        query.bind(now).bind(now).fetch_one(db).await
    }

    pub async fn update(&self, db: &Db, id: i64, values: &Values) -> sqlx::Result<()> {
        let sets: Vec<String> = self
            .columns
            .iter()
            .map(|column| format!("{} = ?", column.name))
            .collect();
        let sql = format!(
            "UPDATE {} SET {}, updated_at = ? WHERE id = ?",
            self.name,
            sets.join(", ")
        );
        let mut query = sqlx::query_scalar::<_, i64>(&sql);
        for column in self.columns {
            query = bind(query, column, values);
        }
        query
            .bind(Utc::now().naive_utc().trunc_subsecs(6))
            .bind(id)
            .fetch_optional(db)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, db: &Db, id: i64) -> sqlx::Result<()> {
        sqlx::query(&format!("DELETE FROM {} WHERE id = ?", self.name))
            .bind(id)
            .execute(db)
            .await?;
        Ok(())
    }
}

type ScalarQuery<'q> = sqlx::query::QueryScalar<'q, sqlx::Sqlite, i64, sqlx::sqlite::SqliteArguments<'q>>;

/// Binds one column's value with its database type. Numbers and booleans
/// that are left empty take the column's default, so a cleared "Pořadí"
/// field saves 0 instead of failing on a NOT NULL column.
fn bind<'q>(query: ScalarQuery<'q>, column: &Column, values: &Values) -> ScalarQuery<'q> {
    let value = values.get(column.name);
    match column.kind {
        Kind::Text => query.bind(value.map(str::to_string)),
        Kind::Integer => query.bind(
            value
                .and_then(cast_integer)
                .or_else(|| column.default.and_then(cast_integer)),
        ),
        Kind::Boolean => query.bind(match value {
            Some(value) if !value.is_empty() => truthy(value),
            _ => column.default == Some("1"),
        }),
    }
}

/// Rails' boolean casting: "0", "f", "false", "off" (any case) are false.
fn truthy(value: &str) -> bool {
    !matches!(value.to_ascii_lowercase().as_str(), "" | "0" | "f" | "false" | "off")
}

/// Rails' integer casting: the leading whole number ("12.7" is 12, "4 m"
/// is 4), or NULL when the text doesn't start with one.
pub fn cast_integer(value: &str) -> Option<i64> {
    let trimmed = value.trim_start();
    let (sign, digits) = match trimmed.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };
    let end = digits.find(|c: char| !c.is_ascii_digit()).unwrap_or(digits.len());
    digits[..end].parse::<i64>().ok().map(|number| sign * number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_and_booleans_are_cast_like_rails() {
        assert_eq!(cast_integer("12"), Some(12));
        assert_eq!(cast_integer(" 12.7"), Some(12));
        assert_eq!(cast_integer("-3 m"), Some(-3));
        assert_eq!(cast_integer("abc"), None);
        assert_eq!(cast_integer(""), None);
        assert!(truthy("1") && truthy("true") && truthy("on"));
        assert!(!truthy("0") && !truthy("FALSE") && !truthy("off"));
    }
}
