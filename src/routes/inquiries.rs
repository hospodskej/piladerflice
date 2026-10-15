//! POST /inquiries: the quote request form on the contact page. The request
//! is saved, the business gets an e-mail, and the visitor goes back to the
//! form with a message.

use axum::extract::State;

use crate::AppState;
use crate::forms::FormData;
use crate::mail;
use crate::models::inquiry::Inquiry;
use crate::models::inquiry_options::{INQUIRY_CATEGORIES, inquiry_row_fields};
use crate::request::Req;
use crate::routes::Result;
use crate::session::Flash;
use crate::views::redirect;

pub async fn create(State(state): State<AppState>, req: Req, form: FormData) -> Result {
    let field = |name: &str| form.get(name).unwrap_or_default().to_string();
    let category = match form.get("kalkulace_category") {
        Some(category) if INQUIRY_CATEGORIES.contains(&category) => category,
        _ => INQUIRY_CATEGORIES[0],
    };
    let inquiry = Inquiry {
        first_name: field("jmeno"),
        last_name: field("prijmeni"),
        email: field("email"),
        phone: field("telefon"),
        street: field("ulice"),
        house_number: field("cislo_popisne"),
        city: field("mesto"),
        zip: field("psc"),
        notes: field("poznamky"),
        category: category.to_string(),
        items_snapshot: items_snapshot(&form, category),
        locale: req.locale.code().to_string(),
        ..Inquiry::default()
    };

    let message = if inquiry.is_valid() {
        let inquiry = inquiry.insert(&state.db).await?;
        match mail::inquiries::new_inquiry(&state, &inquiry) {
            Ok(email) => state.mailer.send(email),
            Err(error) => tracing::error!("e-mail for inquiry #{} not built: {error:#}", inquiry.id),
        }
        Flash::Notice(state.translations.text(req.locale, "kontakt.inquiry_success", &[]))
    } else {
        Flash::Alert(state.translations.text(req.locale, "kontakt.inquiry_error", &[]))
    };
    req.session.set_flash(message);
    Ok(redirect::to(&req.path_for("/kontakt#kalkulace")))
}

/// The item rows as JSON: the form sends each column as a list
/// (`delka[]=4 m&delka[]=5 m`); row n takes the n-th value of each.
fn items_snapshot(form: &FormData, category: &str) -> String {
    let fields = inquiry_row_fields(category);
    let columns: Vec<Vec<&str>> = fields.iter().map(|field| form.all(&format!("{field}[]"))).collect();
    let rows = columns.iter().map(Vec::len).max().unwrap_or(0);
    let items: Vec<serde_json::Map<String, serde_json::Value>> = (0..rows)
        .map(|row| {
            fields
                .iter()
                .zip(&columns)
                .map(|(field, values)| {
                    (
                        field.to_string(),
                        values.get(row).map_or(serde_json::Value::Null, |v| (*v).into()),
                    )
                })
                .collect()
        })
        .collect();
    serde_json::to_string(&items).unwrap_or_else(|_| "[]".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_take_the_nth_value_of_each_column() {
        let form = FormData::parse("polozka%5B%5D=Tr%C3%A1my&delka%5B%5D=4+m&polozka%5B%5D=Prkna".as_bytes());
        let rows: serde_json::Value = serde_json::from_str(&items_snapshot(&form, "stavebni")).unwrap();
        assert_eq!(rows[0]["polozka"], "Trámy");
        assert_eq!(rows[0]["delka"], "4 m");
        assert_eq!(rows[1]["polozka"], "Prkna");
        assert_eq!(rows[1]["delka"], serde_json::Value::Null);
    }
}
