//! /admin/pricelist_items: the price list, one table per category, each
//! row its own little form; plus a row for adding and ↑/↓ for the order.

use axum::extract::State;
use minijinja::context;

use super::params::RecordId;
use crate::AppState;
use crate::admin::pricelist::{self, CATEGORY_FIELDS, FORM_FIELDS, SINGLE_ITEM_CATEGORIES, TABLE};
use crate::admin::record::Values;
use crate::forms::{FormData, UploadForm};
use crate::request::Req;
use crate::routes::{Result, not_found};
use crate::session::Flash;
use crate::views::{admin_layout, redirect};

fn back_to(category: &str) -> String {
    format!("/admin/pricelist_items#{category}")
}

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let ids: Vec<(i64, Option<String>)> =
        sqlx::query_as("SELECT id, category FROM pricelist_items ORDER BY position, id")
            .fetch_all(&state.db)
            .await?;
    let mut categories = Vec::new();
    for (category, fields) in CATEGORY_FIELDS {
        let mut items = Vec::new();
        for (id, _) in ids.iter().filter(|(_, of)| of.as_deref() == Some(category)) {
            if let Some(values) = TABLE.find(&state.db, *id).await? {
                items.push(context! { id => id, values => values });
            }
        }
        let single = SINGLE_ITEM_CATEGORIES.contains(&category);
        categories.push(context! {
            name => category,
            fields => fields,
            movable => !single,
            can_add => !single || items.is_empty(),
            items => items,
        });
    }
    Ok(admin_layout::render(
        &state,
        &req,
        "admin/pricelist_items/index.html",
        context! { categories },
    ))
}

pub async fn create(State(state): State<AppState>, req: Req, form: UploadForm) -> Result {
    let mut values = TABLE.new_record();
    TABLE.assign(&mut values, &form.fields, &FORM_FIELDS);
    let category = values.get("category").unwrap_or_default().to_string();
    let errors = pricelist::validate(&values);
    if !errors.is_empty() {
        req.session.set_flash(Flash::Alert(errors.messages().join(", ")));
        return Ok(redirect::to(&back_to(&category)));
    }
    values.set(
        "position",
        Some(pricelist::next_position(&state.db, &category).await?.to_string()),
    );
    TABLE.insert(&state.db, &values).await?;
    req.session.set_flash(Flash::Notice("Položka byla přidána.".into()));
    Ok(redirect::to(&back_to(&category)))
}

/// POST /admin/pricelist_items/{id}: saves a row.
pub async fn update(State(state): State<AppState>, req: Req, RecordId(id): RecordId, form: UploadForm) -> Result {
    let Some(mut values) = TABLE.find(&state.db, id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    let category = values.get("category").unwrap_or_default().to_string();
    // The category stays: an item doesn't move to another table.
    let editable: Vec<&str> = FORM_FIELDS.into_iter().filter(|name| *name != "category").collect();
    TABLE.assign(&mut values, &form.fields, &editable);
    let errors = pricelist::validate(&values);
    if errors.is_empty() {
        TABLE.update(&state.db, id, &values).await?;
        req.session.set_flash(Flash::Notice("Položka byla uložena.".into()));
    } else {
        req.session.set_flash(Flash::Alert(errors.messages().join(", ")));
    }
    Ok(redirect::to(&back_to(&category)))
}

/// POST /admin/pricelist_items/{id}/delete.
pub async fn delete(State(state): State<AppState>, req: Req, RecordId(id): RecordId, _form: FormData) -> Result {
    let Some(values) = TABLE.find(&state.db, id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    let category = values.get("category").unwrap_or_default().to_string();
    TABLE.delete(&state.db, id).await?;
    req.session.set_flash(Flash::Notice("Položka byla smazána.".into()));
    Ok(redirect::to(&back_to(&category)))
}

pub async fn move_up(state: State<AppState>, req: Req, id: RecordId, _form: UploadForm) -> Result {
    move_item(state, req, id, -1).await
}

pub async fn move_down(state: State<AppState>, req: Req, id: RecordId, _form: UploadForm) -> Result {
    move_item(state, req, id, 1).await
}

async fn move_item(State(state): State<AppState>, req: Req, RecordId(id): RecordId, direction: i64) -> Result {
    let Some(values): Option<Values> = TABLE.find(&state.db, id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    let category = values.get("category").unwrap_or_default().to_string();
    pricelist::move_item(&state.db, id, &category, direction).await?;
    Ok(redirect::to(&back_to(&category)))
}
