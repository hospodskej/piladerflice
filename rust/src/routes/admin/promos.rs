//! /admin/promos: the "Doporučujeme" promos on the home page, which take
//! turns week by week.

use axum::extract::State;
use axum::http::{Method, StatusCode};
use axum::response::Response;
use minijinja::{Value, context};

use super::params::RecordId;
use super::{chosen_image, method_of};
use crate::AppState;
use crate::admin::images;
use crate::admin::promos::{self, PERMITTED, TABLE};
use crate::admin::record::Values;
use crate::czech_time;
use crate::forms::upload::UploadForm;
use crate::models::content::weekly_promo;
use crate::models::upload::image_of;
use crate::models::validation::Errors;
use crate::request::Req;
use crate::routes::{Result, not_found};
use crate::session::Flash;
use crate::views::form_helpers::FormModel;
use crate::views::{admin_layout, redirect};

/// A promo as the list shows it.
#[derive(serde::Serialize, sqlx::FromRow)]
struct Listed {
    id: i64,
    title: Option<String>,
    price: Option<String>,
    link: Option<String>,
}

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let rows: Vec<Listed> = sqlx::query_as("SELECT id, title, price, link FROM promos ORDER BY id")
        .fetch_all(&state.db)
        .await?;
    let this_week = weekly_promo(&state.db, czech_time::today())
        .await?
        .map(|promo| promo.id);
    let promos: Vec<_> = rows
        .iter()
        .map(|promo| context! { this_week => Some(promo.id) == this_week, ..Value::from_serialize(promo) })
        .collect();
    Ok(admin_layout::render(
        &state,
        &req,
        "admin/promos/index.html",
        context! { promos },
    ))
}

pub async fn new(State(state): State<AppState>, req: Req) -> Result {
    Ok(form_page(&state, &req, None, TABLE.new_record(), &Errors::default()).await?)
}

pub async fn create(State(state): State<AppState>, req: Req, form: UploadForm) -> Result {
    let mut values = TABLE.new_record();
    TABLE.assign(&mut values, &form, &PERMITTED);
    let mut errors = Errors::default();
    let image = chosen_image(&form, "promo[image]", &mut errors).await;
    if !errors.is_empty() {
        return Ok(form_page(&state, &req, None, values, &errors).await?);
    }
    let id = TABLE.insert(&state.db, &values).await?;
    if let Some(image) = image {
        images::attach(&state.db, &state.config.storage_root, promos::image(id), &image).await?;
    }
    req.session
        .set_flash(Flash::Notice("Doporučení bylo vytvořeno.".into()));
    Ok(redirect::to("/admin/promos"))
}

pub async fn edit(State(state): State<AppState>, req: Req, RecordId(id): RecordId) -> Result {
    let Some(values) = TABLE.find(&state.db, id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    Ok(form_page(&state, &req, Some(id), values, &Errors::default()).await?)
}

/// POST with `_method`: "patch" saves the form, "delete" deletes.
pub async fn change(
    State(state): State<AppState>,
    req: Req,
    http: Method,
    RecordId(id): RecordId,
    form: UploadForm,
) -> Result {
    let Some(mut values) = TABLE.find(&state.db, id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    match method_of(&http, &form).as_str() {
        "patch" | "put" => {}
        "delete" => {
            promos::delete(&state.db, &state.config.storage_root, id).await?;
            req.session.set_flash(Flash::Notice("Doporučení bylo smazáno.".into()));
            return Ok(redirect::to("/admin/promos"));
        }
        _ => return Ok(not_found::page(State(state), req).await),
    }
    TABLE.assign(&mut values, &form, &PERMITTED);
    let mut errors = Errors::default();
    let image = chosen_image(&form, "promo[image]", &mut errors).await;
    if !errors.is_empty() {
        return Ok(form_page(&state, &req, Some(id), values, &errors).await?);
    }
    TABLE.update(&state.db, id, &values).await?;
    if let Some(image) = image {
        images::attach(&state.db, &state.config.storage_root, promos::image(id), &image).await?;
    }
    req.session.set_flash(Flash::Notice("Doporučení bylo uloženo.".into()));
    Ok(redirect::to("/admin/promos"))
}

async fn form_page(
    state: &AppState,
    req: &Req,
    id: Option<i64>,
    values: Values,
    errors: &Errors,
) -> anyhow::Result<Response> {
    let image = match id {
        Some(id) => image_of(&state.db, "Promo", id).await?.map(|upload| upload.url()),
        None => None,
    };
    let title = values.get("title").unwrap_or_default().to_string();
    let data = context! {
        id => id,
        title => title,
        image => image,
        f => FormModel::new(TABLE.param, values, errors),
    };
    let template = if id.is_some() {
        "admin/promos/edit.html"
    } else {
        "admin/promos/new.html"
    };
    let status = if errors.is_empty() {
        StatusCode::OK
    } else {
        StatusCode::UNPROCESSABLE_ENTITY
    };
    Ok(admin_layout::render_with_status(state, req, status, template, data))
}
