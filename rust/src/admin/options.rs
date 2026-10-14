//! The choices in the drop-down lists of the quote form's calculator
//! (`inquiry_form_options`). The public side is `models/inquiry_options.rs`.

use chrono::{SubsecRound, Utc};

use super::ordering;
use super::record::{Table, Values, integer, text};
use crate::db::Db;
use crate::i18n::Translations;
use crate::locale::Locale;
use crate::models::inquiry_options::INQUIRY_CATEGORIES;
use crate::models::validation::{Errors, Validator};

/// The drop-down fields of each category, in form order.
pub fn dropdown_fields(category: &str) -> &'static [&'static str] {
    match category {
        "palivove" => &["varianta", "druh", "delka", "mnozstvi"],
        "stavebni" => &["polozka", "delka", "vyska", "sirka"],
        _ => &[],
    }
}

pub fn category_label(category: &str) -> Option<&'static str> {
    match category {
        "palivove" => Some("Palivové dřevo"),
        "stavebni" => Some("Stavební řezivo"),
        _ => None,
    }
}

pub fn field_label(field: &str) -> Option<&'static str> {
    Some(match field {
        "varianta" => "Varianta",
        "druh" => "Druh",
        "delka" => "Délka",
        "mnozstvi" => "Množství",
        "polozka" => "Položka",
        "vyska" => "Výška",
        "sirka" => "Šířka",
        _ => return None,
    })
}

pub const TABLE: Table = Table {
    name: "inquiry_form_options",
    param: "inquiry_form_option",
    columns: &[
        text("category"),
        text("field"),
        text("value"),
        text("value_de"),
        integer("position", Some("0")),
    ],
};

pub const PERMITTED: [&str; 4] = ["category", "field", "value", "value_de"];

/// Rails' order: a known category, a value, a field of that category.
pub fn validate(translations: &Translations, values: &Values) -> Errors {
    let mut check = Validator::new(translations, Locale::Cs, "inquiry_form_option");
    let category = values.get("category").unwrap_or_default();
    if !INQUIRY_CATEGORIES.contains(&category) {
        check.add("category", "inclusion");
    }
    if !values.is_present("value") {
        check.add("value", "blank");
    }
    if !dropdown_fields(category).contains(&values.get("field").unwrap_or_default()) {
        check.add("field", "invalid_for_category");
    }
    check.finish()
}

/// "#palivove-varianta": where the option's table is on the page.
pub fn anchor(values: &Values) -> String {
    format!(
        "{}-{}",
        values.get("category").unwrap_or_default(),
        values.get("field").unwrap_or_default()
    )
}

/// The position for a new option: after the last one of its list.
pub async fn next_position(db: &Db, category: &str, field: &str) -> sqlx::Result<i64> {
    let last: Option<i64> =
        sqlx::query_scalar("SELECT MAX(position) FROM inquiry_form_options WHERE category = ? AND field = ?")
            .bind(category)
            .bind(field)
            .fetch_one(db)
            .await?;
    Ok(last.unwrap_or(0) + 1)
}

/// Moves an option up (-1) or down (+1) in its list by swapping positions
/// with its neighbour.
pub async fn move_option(db: &Db, id: i64, values: &Values, direction: i64) -> sqlx::Result<()> {
    let siblings: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT id, position FROM inquiry_form_options WHERE category = ? AND field = ? ORDER BY position, id",
    )
    .bind(values.get("category"))
    .bind(values.get("field"))
    .fetch_all(db)
    .await?;
    let ids: Vec<i64> = siblings.iter().map(|(id, _)| *id).collect();
    let Some((index, target)) = ordering::neighbour(&ids, id, direction) else {
        return Ok(());
    };
    let ((own_id, own_position), (their_id, their_position)) = (siblings[index], siblings[target]);
    let now = Utc::now().naive_utc().trunc_subsecs(6);
    let mut tx = db.begin().await?;
    for (id, position) in [(own_id, their_position), (their_id, own_position)] {
        sqlx::query("UPDATE inquiry_form_options SET position = ?, updated_at = ? WHERE id = ?")
            .bind(position)
            .bind(now)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await
}
