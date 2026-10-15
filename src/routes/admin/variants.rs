//! /admin/catalog_products/{product_id}/catalog_variants: adding, editing
//! and deleting a product's variants. The form shows the fields that the
//! product's layout uses (container/crates for firewood, sizes for lumber).

use axum::extract::State;
use axum::http::{Method, StatusCode};
use axum::response::Response;
use minijinja::context;

use super::params::{ProductId, RecordId};
use super::{chosen_image, method_of};
use crate::AppState;
use crate::admin::images;
use crate::admin::products;
use crate::admin::record::Values;
use crate::admin::variants::{self, PERMITTED, TABLE};
use crate::forms::upload::UploadForm;
use crate::models::upload::{Owner, image_of};
use crate::models::validation::Errors;
use crate::request::Req;
use crate::routes::{Result, not_found};
use crate::session::Flash;
use crate::views::form_helpers::FormModel;
use crate::views::{admin_layout, redirect};

/// The product the variant belongs to.
struct Product {
    id: i64,
    title: String,
    template: String,
}

async fn product(state: &AppState, id: i64) -> sqlx::Result<Option<Product>> {
    Ok(products::TABLE.find(&state.db, id).await?.map(|values| Product {
        id,
        title: values.get("title").unwrap_or_default().to_string(),
        template: values.get("template").unwrap_or_default().to_string(),
    }))
}

/// The variant, if it belongs to the product.
async fn variant_of(state: &AppState, product: &Product, id: i64) -> sqlx::Result<Option<Values>> {
    let variant = TABLE.find(&state.db, id).await?;
    Ok(variant.filter(|values| values.integer("catalog_product_id") == Some(product.id)))
}

pub async fn new(State(state): State<AppState>, req: Req, ProductId(product_id): ProductId) -> Result {
    let Some(product) = product(&state, product_id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    let mut values = TABLE.new_record();
    values.set("catalog_product_id", Some(product.id.to_string()));
    Ok(form_page(&state, &req, &product, None, values, &Errors::default()).await?)
}

pub async fn create(
    State(state): State<AppState>,
    req: Req,
    ProductId(product_id): ProductId,
    form: UploadForm,
) -> Result {
    let Some(product) = product(&state, product_id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    let mut values = TABLE.new_record();
    TABLE.assign(&mut values, &form, &PERMITTED);
    values.set("catalog_product_id", Some(product.id.to_string()));
    let mut errors = variants::validate(&state.db, &state.translations, &values, None).await?;
    let image = chosen_image(&form, "catalog_variant[image]", &mut errors).await;
    if !errors.is_empty() {
        return Ok(form_page(&state, &req, &product, None, values, &errors).await?);
    }
    let id = TABLE.insert(&state.db, &values).await?;
    if let Some(image) = image {
        images::attach(&state.db, &state.config.storage_root, variants::image(id), &image).await?;
    }
    req.session.set_flash(Flash::Notice("Varianta byla přidána.".into()));
    Ok(redirect::to(&format!("/admin/catalog_products/{}/edit", product.id)))
}

pub async fn edit(
    State(state): State<AppState>,
    req: Req,
    ProductId(product_id): ProductId,
    RecordId(id): RecordId,
) -> Result {
    let Some(product) = product(&state, product_id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    let Some(values) = variant_of(&state, &product, id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    Ok(form_page(&state, &req, &product, Some(id), values, &Errors::default()).await?)
}

/// POST with `_method`: "patch" saves the form, "delete" deletes.
pub async fn change(
    State(state): State<AppState>,
    req: Req,
    http: Method,
    ProductId(product_id): ProductId,
    RecordId(id): RecordId,
    form: UploadForm,
) -> Result {
    let Some(product) = product(&state, product_id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    let Some(mut values) = variant_of(&state, &product, id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    let back = format!("/admin/catalog_products/{}/edit", product.id);
    match method_of(&http, &form).as_str() {
        "patch" | "put" => {}
        "delete" => {
            variants::delete(&state.db, &state.config.storage_root, id).await?;
            req.session.set_flash(Flash::Notice("Varianta byla smazána.".into()));
            return Ok(redirect::to(&back));
        }
        _ => return Ok(not_found::page(State(state), req).await),
    }
    TABLE.assign(&mut values, &form, &PERMITTED);
    let mut errors = variants::validate(&state.db, &state.translations, &values, Some(id)).await?;
    let image = chosen_image(&form, "catalog_variant[image]", &mut errors).await;
    if !errors.is_empty() {
        return Ok(form_page(&state, &req, &product, Some(id), values, &errors).await?);
    }
    TABLE.update(&state.db, id, &values).await?;
    if let Some(image) = image {
        images::attach(&state.db, &state.config.storage_root, variants::image(id), &image).await?;
    }
    req.session.set_flash(Flash::Notice("Varianta byla uložena.".into()));
    Ok(redirect::to(&back))
}

/// The new or edit page; with errors (422) when the form didn't save.
async fn form_page(
    state: &AppState,
    req: &Req,
    product: &Product,
    id: Option<i64>,
    values: Values,
    errors: &Errors,
) -> anyhow::Result<Response> {
    let image = match id {
        Some(id) => image_of(&state.db, Owner::Variant, id)
            .await?
            .map(|upload| upload.url()),
        None => None,
    };
    let action = match id {
        Some(id) => format!("/admin/catalog_products/{}/catalog_variants/{id}", product.id),
        None => format!("/admin/catalog_products/{}/catalog_variants", product.id),
    };
    let data = context! {
        id => id,
        action => action,
        product => context! { id => product.id, title => &product.title, template => &product.template },
        image => image,
        f => FormModel::new(TABLE.param, values, errors),
    };
    let template = if id.is_some() {
        "admin/catalog_variants/edit.html"
    } else {
        "admin/catalog_variants/new.html"
    };
    let status = if errors.is_empty() {
        StatusCode::OK
    } else {
        StatusCode::UNPROCESSABLE_ENTITY
    };
    Ok(admin_layout::render_with_status(state, req, status, template, data))
}
