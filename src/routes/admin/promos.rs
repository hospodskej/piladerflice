//! /admin/promos: the "Doporučujeme" promos on the home page, which take
//! turns week by week.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Response;
use leptos::prelude::*;

use super::chosen_image;
use super::params::RecordId;
use crate::AppState;
use crate::admin::images;
use crate::admin::promos::{self, FORM_FIELDS, TABLE};
use crate::admin::record::Values;
use crate::czech_time;
use crate::forms::Errors;
use crate::forms::{FormData, UploadForm};
use crate::models::content::weekly_promo;
use crate::models::upload::{Owner, image_of};
use crate::request::Req;
use crate::routes::{Result, not_found};
use crate::session::Flash;
use crate::ui::admin;
use crate::ui::admin::form::Form;
use crate::ui::admin::promos::{PromoForm, PromoPage, PromoRow, PromosPage};
use crate::views::redirect;

/// A promo as the list shows it.
#[derive(sqlx::FromRow)]
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
    let promos: Vec<PromoRow> = rows
        .into_iter()
        .map(|promo| PromoRow {
            this_week: Some(promo.id) == this_week,
            id: promo.id,
            title: promo.title,
            price: promo.price,
            link: promo.link,
        })
        .collect();
    Ok(admin::page(
        &state,
        &req,
        StatusCode::OK,
        move || view! { <PromosPage promos/> },
    ))
}

pub async fn new(State(state): State<AppState>, req: Req) -> Result {
    Ok(form_page(&state, &req, None, TABLE.new_record(), &Errors::default()).await?)
}

pub async fn create(State(state): State<AppState>, req: Req, form: UploadForm) -> Result {
    let mut values = TABLE.new_record();
    TABLE.assign(&mut values, &form.fields, &FORM_FIELDS);
    let mut errors = Errors::default();
    let image = chosen_image(&form, "image", &mut errors).await;
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

/// POST /admin/promos/{id}: saves the form.
pub async fn update(State(state): State<AppState>, req: Req, RecordId(id): RecordId, form: UploadForm) -> Result {
    let Some(mut values) = TABLE.find(&state.db, id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    TABLE.assign(&mut values, &form.fields, &FORM_FIELDS);
    let mut errors = Errors::default();
    let image = chosen_image(&form, "image", &mut errors).await;
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

/// POST /admin/promos/{id}/delete.
pub async fn delete(State(state): State<AppState>, req: Req, RecordId(id): RecordId, _form: FormData) -> Result {
    if TABLE.find(&state.db, id).await?.is_none() {
        return Ok(not_found::page(State(state), req).await);
    }
    promos::delete(&state.db, &state.config.storage_root, id).await?;
    req.session.set_flash(Flash::Notice("Doporučení bylo smazáno.".into()));
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
        Some(id) => image_of(&state.db, Owner::Promo, id).await?.map(|upload| upload.url()),
        None => None,
    };
    let promo = PromoForm {
        id,
        title: values.get("title").unwrap_or_default().to_string(),
        image,
        f: Form::new(values, errors),
    };
    let status = if errors.is_empty() {
        StatusCode::OK
    } else {
        StatusCode::UNPROCESSABLE_ENTITY
    };
    Ok(admin::page(state, req, status, move || view! { <PromoPage promo/> }))
}
