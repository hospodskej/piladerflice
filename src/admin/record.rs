//! Editing a table row from a form.
//!
//! Each editable table is described once as a [`Table`]: its columns and
//! what kind each is. A record's values are kept as the text a form shows
//! ([`Values`]), so a form that didn't save shows exactly what was typed;
//! numbers and true/false are converted when the record is written. A
//! number field that holds something else is an error on the form
//! ([`Table::check_numbers`]).

use std::collections::BTreeMap;

use chrono::Utc;
use serde::Serialize;
use sqlx::Row;

use crate::db::Db;
use crate::forms::{Errors, FormData};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Text,
    /// A whole number; the label names it in messages ("Pořadí").
    Integer {
        label: &'static str,
    },
    Boolean,
}

#[derive(Clone, Copy, Debug)]
pub struct Column {
    pub name: &'static str,
    pub kind: Kind,
    /// The value a new record starts with, and what an emptied number or
    /// true/false field saves.
    pub default: Option<&'static str>,
}

pub const fn text(name: &'static str) -> Column {
    Column {
        name,
        kind: Kind::Text,
        default: None,
    }
}

pub const fn integer(name: &'static str, label: &'static str, default: Option<&'static str>) -> Column {
    Column {
        name,
        kind: Kind::Integer { label },
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

/// An editable table.
pub struct Table {
    pub name: &'static str,
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

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(String::as_str)
    }

    /// A checkbox's state.
    pub fn is_true(&self, name: &str) -> bool {
        self.get(name).is_some_and(sapwood_forms::de::is_true)
    }

    /// A number field's number; None when it is empty or not a number.
    pub fn integer(&self, name: &str) -> Option<i64> {
        self.get(name).and_then(parse_integer)
    }

    /// Filled in, not just spaces.
    pub fn is_present(&self, name: &str) -> bool {
        self.get(name).is_some_and(|value| !value.trim().is_empty())
    }
}

/// "12" or " 12 " is 12; "12.5", "12 kg" and "" are not whole numbers.
pub fn parse_integer(value: &str) -> Option<i64> {
    value.trim().parse().ok()
}

impl Table {
    fn column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|column| column.name == name)
    }

    /// A new record: the defaults, everything else empty.
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

    /// Takes the submitted `fields` (the ones this form edits; anything
    /// else sent is ignored). A checkbox that wasn't ticked sends nothing,
    /// which is false; other fields not sent stay as they were.
    pub fn assign(&self, values: &mut Values, form: &FormData, fields: &[&str]) {
        for name in fields {
            let Some(column) = self.column(name) else {
                continue;
            };
            match (column.kind, form.get(name)) {
                (_, Some(value)) => values.set(name, Some(value.to_string())),
                (Kind::Boolean, None) => values.set(name, Some("0".into())),
                (_, None) => {}
            }
        }
    }

    /// "Pořadí musí být celé číslo" for each number field holding text
    /// (fields that already have an error are left alone).
    pub fn check_numbers(&self, values: &Values, errors: &mut Errors) {
        for column in self.columns {
            if let Kind::Integer { label } = column.kind
                && !errors.has(column.name)
                && values.is_present(column.name)
                && values.integer(column.name).is_none()
            {
                errors.add(column.name, format!("{label} musí být celé číslo"));
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
        let now = Utc::now().naive_utc();
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
        query.bind(Utc::now().naive_utc()).bind(id).fetch_optional(db).await?;
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

/// Binds one column's value with its database type. Numbers and true/false
/// left empty take the column's default, so a cleared "Pořadí" saves 0.
fn bind<'q>(query: ScalarQuery<'q>, column: &Column, values: &Values) -> ScalarQuery<'q> {
    let value = values.get(column.name);
    match column.kind {
        Kind::Text => query.bind(value.map(str::to_string)),
        Kind::Integer { .. } => query.bind(
            value
                .and_then(parse_integer)
                .or_else(|| column.default.and_then(parse_integer)),
        ),
        Kind::Boolean => query.bind(match value {
            Some(value) if !value.is_empty() => sapwood_forms::de::is_true(value),
            _ => column.default == Some("1"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLE: Table = Table {
        name: "things",
        columns: &[
            text("title"),
            integer("position", "Pořadí", Some("0")),
            boolean("active", true),
        ],
    };

    #[test]
    fn numbers_must_be_whole_numbers() {
        assert_eq!(parse_integer(" 12 "), Some(12));
        assert_eq!(parse_integer("-3"), Some(-3));
        assert_eq!(parse_integer("12.7"), None);
        assert_eq!(parse_integer("4 m"), None);
        assert_eq!(parse_integer(""), None);

        let mut values = TABLE.new_record();
        values.set("position", Some("4 m".into()));
        let mut errors = Errors::new();
        TABLE.check_numbers(&values, &mut errors);
        assert_eq!(errors.messages(), ["Pořadí musí být celé číslo"]);
    }

    #[test]
    fn a_form_sets_its_fields_and_unticked_boxes_are_false() {
        let mut values = TABLE.new_record();
        values.set("title", Some("Old".into()));
        TABLE.assign(
            &mut values,
            &FormData::parse(b"position=3&title=New"),
            &["position", "active"],
        );
        assert_eq!(values.get("title"), Some("Old"), "not a field of this form");
        assert_eq!(values.get("position"), Some("3"));
        assert!(!values.is_true("active"));
        TABLE.assign(&mut values, &FormData::parse(b"active=1"), &["active"]);
        assert!(values.is_true("active"));
    }
}
