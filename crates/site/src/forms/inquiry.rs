//! The inquiry form on the contact page.

use super::{field, locale_of, redirect_with, Fields};
use crate::i18n::{t, Locale};
use crate::pages::kontakt::ROW_FIELDS;
use crate::state::AppState;
use crate::visitor::Flash;
use axum::extract::State;
use axum::http::Uri;
use axum::response::Response;
use axum::Form;
use serde_json::{Map, Value};
use tower_sessions::Session;

/// The rows asked for, per category: {"palivove": [{"druh": "Buk", ...}], ...}.
/// A category is left out when its part of the form was switched off.
pub fn items_snapshot(fields: &Fields) -> Map<String, Value> {
    let mut snapshot = Map::new();
    for (category, names) in ROW_FIELDS {
        let columns: Vec<(&str, Vec<&str>)> = names
            .iter()
            .map(|name| {
                let key = format!("items[{category}][{name}][]");
                (*name, fields.iter().filter(|(k, _)| *k == key).map(|(_, v)| v.as_str()).collect())
            })
            .collect();
        let row_count = columns.iter().map(|(_, values)| values.len()).max().unwrap_or(0);
        let rows: Vec<Value> = (0..row_count)
            .map(|i| {
                Value::Object(
                    columns
                        .iter()
                        .map(|(name, values)| ((*name).to_owned(), values.get(i).map_or(Value::Null, |v| Value::String((*v).to_owned()))))
                        .collect(),
                )
            })
            .collect();
        if !rows.is_empty() {
            snapshot.insert(category.to_owned(), Value::Array(rows));
        }
    }
    snapshot
}

/// The Czech name of a category, for the e-mail.
fn category_label(category: &str) -> &'static str {
    match category {
        "palivove" => "Palivové dřevo",
        _ => "Stavební řezivo",
    }
}

pub async fn create(State(state): State<AppState>, session: Session, uri: Uri, Form(fields): Form<Fields>) -> Response {
    let l = locale_of(&uri);
    let items = items_snapshot(&fields);
    let required = ["jmeno", "prijmeni", "email", "telefon"].map(|name| field(&fields, name));
    let ([Some(first_name), Some(last_name), Some(email), Some(phone)], false) = (required, items.is_empty()) else {
        return redirect_with(&session, l, "/kontakt#kalkulace", Some(Flash::Alert(t::kontakt::inquiry_error(l).to_owned()))).await;
    };
    let (street, house_number, city, zip, notes) =
        (field(&fields, "ulice"), field(&fields, "cislo_popisne"), field(&fields, "mesto"), field(&fields, "psc"), field(&fields, "poznamky"));
    let items_json = Value::Object(items.clone()).to_string();
    let locale = l.as_str();
    let saved = sqlx::query_scalar!(
        r#"INSERT INTO inquiries (locale, items, first_name, last_name, email, phone, street, house_number, city, zip, notes)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id AS "id!""#,
        locale, items_json, first_name, last_name, email, phone, street, house_number, city, zip, notes
    )
    .fetch_one(&state.db)
    .await;
    let id = match saved {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("inquiry not saved: {e}");
            return redirect_with(&session, l, "/kontakt#kalkulace", Some(Flash::Alert(t::kontakt::inquiry_error(l).to_owned()))).await;
        }
    };

    // The sawmill hears about it in Czech.
    if let Some(to) = crate::mail::notification_address(true) {
        let full_name = format!("{first_name} {last_name}");
        let address = [
            [street, house_number].into_iter().flatten().collect::<Vec<_>>().join(" "),
            [zip, city].into_iter().flatten().collect::<Vec<_>>().join(" "),
        ]
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(", ");
        let mut text = format!(
            "NOVÁ POPTÁVKA #{id}\n{} (jazyk formuláře: {})\n\nKONTAKT\nJméno: {full_name}\nE-mail: {email}\nTelefon: {phone}\n",
            crate::time_fmt::now_cz(),
            if l == Locale::De { "němčina" } else { "čeština" }
        );
        if !address.is_empty() {
            text.push_str(&format!("Adresa: {address}\n"));
        }
        for (category, names) in ROW_FIELDS {
            let Some(Value::Array(rows)) = items.get(category) else { continue };
            text.push_str(&format!("\nPOŽADOVANÉ POLOŽKY – {}\n", category_label(category).to_uppercase()));
            for row in rows {
                let cells: Vec<&str> = names.iter().map(|n| row.get(*n).and_then(Value::as_str).unwrap_or_default()).collect();
                text.push_str(&format!("- {}\n", cells.join(" | ")));
            }
        }
        if let Some(notes) = notes {
            text.push_str(&format!("\n\nPOZNÁMKA\n{notes}\n"));
        }
        tokio::spawn(crate::mail::send(crate::mail::Mail {
            to,
            subject: format!("Nová poptávka #{id} – {full_name}"),
            text,
            html: None,
        }));
    }
    redirect_with(&session, l, "/kontakt#kalkulace", Some(Flash::Notice(t::kontakt::inquiry_success(l).to_owned()))).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_from_columns() {
        let fields: Fields = vec![
            ("items[palivove][varianta][]".into(), "Volně".into()),
            ("items[palivove][druh][]".into(), "Buk".into()),
            ("items[palivove][delka][]".into(), "33 cm".into()),
            ("items[palivove][mnozstvi][]".into(), "5 PRM".into()),
            ("items[palivove][varianta][]".into(), "Skládané".into()),
            ("items[palivove][druh][]".into(), "Dub".into()),
            ("items[palivove][delka][]".into(), "1 m".into()),
            ("items[palivove][mnozstvi][]".into(), "2 PRM".into()),
        ];
        let snapshot = items_snapshot(&fields);
        assert!(snapshot.get("stavebni").is_none());
        let rows = snapshot["palivove"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1]["druh"], "Dub");
        assert_eq!(rows[0]["mnozstvi"], "5 PRM");
    }
}
