//! /admin/inquiry_form_options: the choices of the quote form's drop-down
//! lists, one table per list; the first choice is preselected.

use axum::extract::State;
use minijinja::{Value, context};

use super::params::RecordId;
use crate::AppState;
use crate::admin::messages::sentence;
use crate::admin::options::{self, FORM_FIELDS, TABLE, category_label, dropdown_fields, field_label};
use crate::forms::{FormData, UploadForm};
use crate::models::inquiry_options::INQUIRY_CATEGORIES;
use crate::request::Req;
use crate::routes::{Result, not_found};
use crate::session::Flash;
use crate::views::{admin_layout, redirect};

const PAGE: &str = "/admin/inquiry_form_options";

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let rows: Vec<(i64, String, String, String, Option<String>)> =
        sqlx::query_as("SELECT id, category, field, value, value_de FROM inquiry_form_options ORDER BY position, id")
            .fetch_all(&state.db)
            .await?;
    let categories: Vec<Value> = INQUIRY_CATEGORIES
        .iter()
        .map(|category| {
            let lists: Vec<Value> = dropdown_fields(category)
                .iter()
                .map(|field| {
                    let choices: Vec<Value> = rows
                        .iter()
                        .filter(|(_, of_category, of_field, _, _)| of_category == category && of_field == field)
                        .map(|(id, _, _, value, value_de)| context! { id, value, value_de })
                        .collect();
                    context! { field, label => field_label(field), choices }
                })
                .collect();
            context! { name => category, label => category_label(category), lists }
        })
        .collect();
    Ok(admin_layout::render(
        &state,
        &req,
        "admin/inquiry_form_options/index.html",
        context! { categories },
    ))
}

pub async fn create(State(state): State<AppState>, req: Req, form: UploadForm) -> Result {
    let mut values = TABLE.new_record();
    TABLE.assign(&mut values, &form.fields, &FORM_FIELDS);
    let errors = options::validate(&values);
    if !errors.is_empty() {
        let message = sentence(&errors.messages());
        req.session.set_flash(Flash::Alert(message));
        return Ok(redirect::to(PAGE));
    }
    let position = options::next_position(
        &state.db,
        values.get("category").unwrap_or_default(),
        values.get("field").unwrap_or_default(),
    )
    .await?;
    values.set("position", Some(position.to_string()));
    TABLE.insert(&state.db, &values).await?;
    req.session.set_flash(Flash::Notice("Možnost byla přidána.".into()));
    Ok(redirect::to(&format!("{PAGE}#{}", options::anchor(&values))))
}

/// POST /admin/inquiry_form_options/{id}: saves an option.
pub async fn update(State(state): State<AppState>, req: Req, RecordId(id): RecordId, form: UploadForm) -> Result {
    let Some(mut values) = TABLE.find(&state.db, id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    // An option stays in its list.
    TABLE.assign(&mut values, &form.fields, &["value", "value_de"]);
    let errors = options::validate(&values);
    if errors.is_empty() {
        TABLE.update(&state.db, id, &values).await?;
        req.session.set_flash(Flash::Notice("Možnost byla uložena.".into()));
    } else {
        let message = sentence(&errors.messages());
        req.session.set_flash(Flash::Alert(message));
    }
    Ok(redirect::to(&format!("{PAGE}#{}", options::anchor(&values))))
}

/// POST /admin/inquiry_form_options/{id}/delete.
pub async fn delete(State(state): State<AppState>, req: Req, RecordId(id): RecordId, _form: FormData) -> Result {
    if TABLE.find(&state.db, id).await?.is_none() {
        return Ok(not_found::page(State(state), req).await);
    }
    TABLE.delete(&state.db, id).await?;
    req.session.set_flash(Flash::Notice("Možnost byla smazána.".into()));
    Ok(redirect::to(PAGE))
}

pub async fn move_up(state: State<AppState>, req: Req, id: RecordId, _form: UploadForm) -> Result {
    move_option(state, req, id, -1).await
}

pub async fn move_down(state: State<AppState>, req: Req, id: RecordId, _form: UploadForm) -> Result {
    move_option(state, req, id, 1).await
}

async fn move_option(State(state): State<AppState>, req: Req, RecordId(id): RecordId, direction: i64) -> Result {
    let Some(values) = TABLE.find(&state.db, id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    options::move_option(&state.db, id, &values, direction).await?;
    Ok(redirect::to(&format!("{PAGE}#{}", options::anchor(&values))))
}
