//! The notification of a new quote request, for the business (Czech).

use minijinja::context;

use super::Email;
use super::orders::render;
use crate::AppState;
use crate::models::inquiry::Inquiry;
use crate::models::inquiry_options::inquiry_row_fields;

fn category_label(category: &str) -> &str {
    match category {
        "palivove" => "Palivové dřevo",
        "stavebni" => "Stavební řezivo",
        other => other,
    }
}

fn field_label(field: &str) -> &'static str {
    match field {
        "varianta" => "Varianta",
        "druh" => "Druh",
        "delka" => "Délka",
        "mnozstvi" => "Množství",
        "polozka" => "Položka",
        "vyska" => "Výška",
        "sirka" => "Šířka",
        _ => "",
    }
}

pub fn new_inquiry(state: &AppState, inquiry: &Inquiry) -> anyhow::Result<Email> {
    let fields = inquiry_row_fields(&inquiry.category);
    let rows: Vec<Vec<String>> = inquiry
        .items()
        .iter()
        .map(|item| {
            fields
                .iter()
                .map(|field| match item.get(*field) {
                    Some(serde_json::Value::String(text)) => text.clone(),
                    Some(serde_json::Value::Null) | None => String::new(),
                    Some(other) => other.to_string(),
                })
                .collect()
        })
        .collect();
    let data = context! {
        locale => "cs",
        id => inquiry.id,
        created_at => inquiry.created_at.format("%d.%m.%Y %H:%M").to_string(),
        german => inquiry.locale == "de",
        full_name => inquiry.full_name(),
        email => &inquiry.email,
        phone => &inquiry.phone,
        address => inquiry.full_address(),
        category => category_label(&inquiry.category),
        headers => fields.iter().map(|field| field_label(field)).collect::<Vec<_>>(),
        rows,
        notes => &inquiry.notes,
    };
    let subject = format!("Nová poptávka #{} – {}", inquiry.id, inquiry.full_name());
    render(
        state,
        "new_inquiry",
        state.mail_settings.inquiry_notification_to.clone(),
        subject,
        data,
    )
}
