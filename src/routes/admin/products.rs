//! /admin/catalog_products: the catalogue, adding and editing products
//! (with their variants listed below the form), deleting them.

use axum::extract::State;
use axum::http::{Method, StatusCode};
use axum::response::Response;
use minijinja::{Value, context};

use super::params::RecordId;
use super::{chosen_image, czk, method_of};
use crate::AppState;
use crate::admin::images;
use crate::admin::products::{self, PERMITTED, TABLE};
use crate::admin::record::Values;
use crate::admin::variants;
use crate::forms::upload::UploadForm;
use crate::models::upload::image_of;
use crate::models::validation::Errors;
use crate::request::Req;
use crate::routes::{Result, not_found};
use crate::session::Flash;
use crate::views::form_helpers::FormModel;
use crate::views::{admin_layout, redirect};

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let products: Vec<Value> = products::list(&state.db)
        .await?
        .into_iter()
        .map(|product| {
            context! {
                starting_price => product.starting_price_czk.map(czk),
                ..Value::from_serialize(&product)
            }
        })
        .collect();
    Ok(admin_layout::render(
        &state,
        &req,
        "admin/catalog_products/index.html",
        context! { products },
    ))
}

pub async fn new(State(state): State<AppState>, req: Req) -> Result {
    Ok(form_page(&state, &req, None, TABLE.new_record(), &Errors::default()).await?)
}

pub async fn create(State(state): State<AppState>, req: Req, form: UploadForm) -> Result {
    let mut values = TABLE.new_record();
    TABLE.assign(&mut values, &form, &PERMITTED);
    products::normalize(&mut values);
    let mut errors = products::validate(&state.db, &state.translations, &values, None).await?;
    let image = chosen_image(&form, "catalog_product[image]", &mut errors).await;
    if !errors.is_empty() {
        return Ok(form_page(&state, &req, None, values, &errors).await?);
    }
    let id = TABLE.insert(&state.db, &values).await?;
    if let Some(image) = image {
        images::attach(&state.db, &state.config.storage_root, products::image(id), &image).await?;
    }
    req.session.set_flash(Flash::Notice(
        "Produkt byl vytvořen. Nyní k němu přidejte varianty s cenami.".into(),
    ));
    Ok(redirect::to(&format!("/admin/catalog_products/{id}/edit")))
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
            products::delete(&state.db, &state.config.storage_root, id).await?;
            req.session.set_flash(Flash::Notice("Produkt byl smazán.".into()));
            return Ok(redirect::to("/admin/catalog_products"));
        }
        _ => return Ok(not_found::page(State(state), req).await),
    }
    TABLE.assign(&mut values, &form, &PERMITTED);
    products::normalize(&mut values);
    let mut errors = products::validate(&state.db, &state.translations, &values, Some(id)).await?;
    let image = chosen_image(&form, "catalog_product[image]", &mut errors).await;
    if !errors.is_empty() {
        return Ok(form_page(&state, &req, Some(id), values, &errors).await?);
    }
    TABLE.update(&state.db, id, &values).await?;
    if let Some(image) = image {
        images::attach(&state.db, &state.config.storage_root, products::image(id), &image).await?;
    }
    req.session.set_flash(Flash::Notice("Produkt byl uložen.".into()));
    Ok(redirect::to(&format!("/admin/catalog_products/{id}/edit")))
}

/// The new or edit page; with errors (422) when the form didn't save.
async fn form_page(
    state: &AppState,
    req: &Req,
    id: Option<i64>,
    values: Values,
    errors: &Errors,
) -> anyhow::Result<Response> {
    let image = match id {
        Some(id) => image_of(&state.db, "CatalogProduct", id)
            .await?
            .map(|upload| upload.url()),
        None => None,
    };
    let variants = match id {
        Some(id) => variant_rows(state, id, values.get("template").unwrap_or_default()).await?,
        None => Vec::new(),
    };
    let title = values.get("title").unwrap_or_default().to_string();
    let data = context! {
        id => id,
        title => title,
        image => image,
        variants => variants,
        f => FormModel::new(TABLE.param, values, errors),
    };
    let template = if id.is_some() {
        "admin/catalog_products/edit.html"
    } else {
        "admin/catalog_products/new.html"
    };
    let status = if errors.is_empty() {
        StatusCode::OK
    } else {
        StatusCode::UNPROCESSABLE_ENTITY
    };
    Ok(admin_layout::render_with_status(state, req, status, template, data))
}

/// The product's variants as the table below the form shows them.
async fn variant_rows(state: &AppState, product_id: i64, template: &str) -> anyhow::Result<Vec<Value>> {
    let images = crate::models::upload::images_of(&state.db, "CatalogVariant").await?;
    let ids: Vec<i64> =
        sqlx::query_scalar("SELECT id FROM catalog_variants WHERE catalog_product_id = ? ORDER BY position, id")
            .bind(product_id)
            .fetch_all(&state.db)
            .await?;
    let mut rows = Vec::new();
    for id in ids {
        let Some(variant) = variants::TABLE.find(&state.db, id).await? else {
            continue;
        };
        let description = if template == "lumber" {
            let size = match (variant.get("width_mm"), variant.get("height_mm")) {
                (Some(width), Some(height)) if !width.is_empty() && !height.is_empty() => {
                    Some(format!("{width}×{height} mm"))
                }
                _ => None,
            };
            [variant.get("grade").map(str::to_string), size]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" / ")
        } else {
            variant.get("variant_label").unwrap_or_default().to_string()
        };
        let group = variant.is_present("variant_group").then(|| {
            if variant.get("variant_group") == Some("bedny") {
                "Skládané (bedny)"
            } else if variants::is_stacked(&variant) {
                "Skládané (kontejner)"
            } else {
                "Sypané (kontejner)"
            }
        });
        rows.push(context! {
            id => id,
            image => images.get(&id).map(|upload| upload.url()),
            description => description,
            group => group,
            length_label => variant.get("length_label"),
            amount_value => variant.get("amount_value"),
            price => variant.integer("price_czk").map(czk),
            in_stock => variant.is_true("in_stock"),
        });
    }
    Ok(rows)
}
