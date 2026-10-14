//! The contact page: address and hours, the quote request form (Kalkulace)
//! and the price list (Ceník).

mod price_list;
mod quote_form;

use axum::extract::State;
use minijinja::context;

use super::Result;
use super::components::business_schema;
use crate::AppState;
use crate::request::Req;
use crate::views::{self, Page};

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let money = state.rates.money(req.locale).await;
    let prices = price_list::load(&state, &money).await?;
    let form_sections = quote_form::sections(&state, req.locale).await?;

    let mut page = Page::meta(&state, &req, "kontakt");
    page.head_html = Some(business_schema::tag(&state, &req));
    Ok(views::render(
        &state,
        &req,
        "home/kontakt.html",
        page,
        context! { form_sections, ..prices },
    ))
}
