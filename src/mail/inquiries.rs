//! The notification of a new quote request, for the business (Czech). The
//! HTML is `ui::mail`; the plain text is written here.

use std::fmt::Write;

use leptos::prelude::*;

use super::{Email, render_html};
use crate::AppState;
use crate::locale::Locale;
use crate::models::inquiry::Inquiry;
use crate::models::inquiry_options::inquiry_row_fields;
use crate::ui::mail::NewInquiry;

/// A quote request as its e-mail shows it.
#[derive(Clone)]
pub struct InquiryMail {
    pub id: i64,
    pub created_at: String,
    pub german: bool,
    pub full_name: String,
    pub email: String,
    pub phone: String,
    /// Empty without one.
    pub address: String,
    /// "Stavební řezivo"
    pub category: String,
    /// The items' columns, and each item's values in that order.
    pub headers: Vec<&'static str>,
    pub rows: Vec<Vec<String>>,
    pub notes: String,
}

impl InquiryMail {
    pub fn language(&self) -> &'static str {
        if self.german { "němčina" } else { "čeština" }
    }
}

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
    let mail = InquiryMail {
        id: inquiry.id,
        created_at: crate::czech_time::from_utc(inquiry.created_at)
            .format("%d.%m.%Y %H:%M")
            .to_string(),
        german: inquiry.locale == "de",
        full_name: inquiry.full_name(),
        email: inquiry.email.clone(),
        phone: inquiry.phone.clone(),
        address: inquiry.full_address(),
        category: category_label(&inquiry.category).to_string(),
        headers: fields.iter().map(|field| field_label(field)).collect(),
        rows: inquiry
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
            .collect(),
        notes: inquiry.notes.clone(),
    };
    let subject = format!("Nová poptávka #{} – {}", inquiry.id, inquiry.full_name());
    let text = text(&mail);
    let html = render_html(state, Locale::Cs, move || view! { <NewInquiry inquiry=mail/> });
    Ok(Email {
        to: state.mail_settings.inquiry_notification_to.clone(),
        subject,
        html,
        text,
    })
}

fn text(mail: &InquiryMail) -> String {
    let mut text = String::new();
    let _ = writeln!(
        text,
        "NOVÁ POPTÁVKA #{}\n{} (jazyk formuláře: {})\n",
        mail.id,
        mail.created_at,
        mail.language()
    );
    let _ = writeln!(
        text,
        "KONTAKT\nJméno: {}\nE-mail: {}\nTelefon: {}",
        mail.full_name, mail.email, mail.phone
    );
    if !mail.address.is_empty() {
        let _ = writeln!(text, "Adresa: {}", mail.address);
    }
    let _ = writeln!(text, "\nPOŽADOVANÉ POLOŽKY ({})", mail.category);
    for row in &mail.rows {
        let _ = writeln!(text, "- {}", row.join(" | "));
    }
    if !mail.notes.trim().is_empty() {
        let _ = write!(text, "\nPOZNÁMKA\n{}\n", mail.notes);
    }
    text
}
