//! The contact page: address and hours, the quote request form (Kalkulace)
//! and the price list (Ceník).

mod price_list;
mod quote_form;

use axum::extract::State;
use leptos::prelude::*;

use super::Result;
use super::components::business_schema;
use crate::AppState;
use crate::request::Req;
use crate::ui::{self, contact::ContactPage};
use crate::views::Page;

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let money = state.rates.money(req.locale).await;
    let prices = price_list::load(&state, &money).await?;
    let form_sections = quote_form::sections(&state, req.locale).await?;

    let mut page = Page::meta(&state, &req, "kontakt");
    page.json_ld = Some(business_schema::json(&state, &req));
    Ok(ui::render::page(
        &state,
        &req,
        page,
        move || view! { <ContactPage form_sections prices/> },
    )
    .await)
}
