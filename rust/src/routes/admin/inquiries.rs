//! /admin/inquiries: the quote requests from the contact page.

use axum::extract::State;
use minijinja::{Value, context};

use super::date;
use super::params::RecordId;
use crate::AppState;
use crate::admin::options::{category_label, field_label};
use crate::models::inquiry::Inquiry;
use crate::models::inquiry_options::inquiry_row_fields;
use crate::request::Req;
use crate::routes::{Result, not_found};
use crate::views::admin_layout;

fn category(inquiry: &Inquiry) -> String {
    category_label(&inquiry.category).map_or_else(|| inquiry.category.clone(), str::to_string)
}

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let inquiries: Vec<Value> = Inquiry::latest(&state.db)
        .await?
        .iter()
        .map(|inquiry| {
            context! {
                id => inquiry.id,
                full_name => inquiry.full_name(),
                email => &inquiry.email,
                phone => &inquiry.phone,
                category => category(inquiry),
                item_count => inquiry.items().len(),
                date => date(&inquiry.created_at),
            }
        })
        .collect();
    Ok(admin_layout::render(
        &state,
        &req,
        "admin/inquiries/index.html",
        context! { inquiries },
    ))
}

pub async fn show(State(state): State<AppState>, req: Req, RecordId(id): RecordId) -> Result {
    let Some(inquiry) = Inquiry::find(&state.db, id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    let fields = if crate::models::inquiry_options::INQUIRY_CATEGORIES.contains(&inquiry.category.as_str()) {
        inquiry_row_fields(&inquiry.category)
    } else {
        &[]
    };
    // Each item's values in column order; a missing value is empty, numbers
    // print as numbers.
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
        inquiry => context! {
            id => inquiry.id,
            date => date(&inquiry.created_at),
            german => inquiry.locale == "de",
            full_name => inquiry.full_name(),
            email => &inquiry.email,
            phone => &inquiry.phone,
            full_address => inquiry.full_address(),
            category => category(&inquiry),
            notes => &inquiry.notes,
        },
        headers => fields.iter().map(|field| field_label(field).unwrap_or_default()).collect::<Vec<_>>(),
        rows => rows,
    };
    Ok(admin_layout::render(&state, &req, "admin/inquiries/show.html", data))
}
