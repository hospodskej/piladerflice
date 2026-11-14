//! The checkout steps' forms.

use super::{Fields, locale_of};
use crate::session::{CHECKOUT_ERRORS, LAST_ORDER, save_checkout};
use crate::state::AppState;
use crate::visitor::Visitor;
use axum::extract::State;
use axum::http::Uri;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Form};
use tower_sessions::Session;

/// The last value of a field (a ticked checkbox comes after its hidden "0").
fn last(fields: &Fields) -> impl Fn(&str) -> Option<String> + '_ {
    |name| {
        fields
            .iter()
            .rev()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
    }
}

pub async fn update_shipping(
    session: Session,
    Extension(mut v): Extension<Visitor>,
    uri: Uri,
    Form(fields): Form<Fields>,
) -> Response {
    let l = locale_of(&uri);
    if v.cart.is_empty() {
        return Redirect::to(&l.path("/kosik")).into_response();
    }
    v.checkout.update_shipping(last(&fields));
    if let Err(e) = save_checkout(&session, &v.checkout).await {
        tracing::warn!("checkout not saved: {e}");
    }
    Redirect::to(&l.path("/kosik/udaje")).into_response()
}

pub async fn update_details(
    session: Session,
    Extension(mut v): Extension<Visitor>,
    uri: Uri,
    Form(fields): Form<Fields>,
) -> Response {
    let l = locale_of(&uri);
    if v.cart.is_empty() {
        return Redirect::to(&l.path("/kosik")).into_response();
    }
    v.checkout.update_details(last(&fields));
    if let Err(e) = save_checkout(&session, &v.checkout).await {
        tracing::warn!("checkout not saved: {e}");
    }
    Redirect::to(&l.path("/kosik/souhrn")).into_response()
}

pub async fn confirm(
    State(state): State<AppState>,
    session: Session,
    ip: refractory::RemoteIp,
    Extension(mut v): Extension<Visitor>,
    uri: Uri,
    Form(fields): Form<Fields>,
) -> Response {
    let l = locale_of(&uri);
    if let Some(refused) = super::too_many_attempts(
        &state,
        &session,
        &ip,
        &crate::limits::ORDER,
        l,
        "/kosik/souhrn",
    )
    .await
    {
        return refused;
    }
    if v.cart.is_empty() {
        return Redirect::to(&l.path("/kosik")).into_response();
    }
    if !v.checkout.shipping_selected() {
        return Redirect::to(&l.path("/kosik/doprava")).into_response();
    }
    if !v.checkout.details_complete() {
        return Redirect::to(&l.path("/kosik/udaje")).into_response();
    }
    let refuse_all = |messages: Vec<String>| async {
        let _ = session.insert(CHECKOUT_ERRORS, messages).await;
        Redirect::to(&l.path("/kosik/souhrn")).into_response()
    };
    let refuse = |message: String| refuse_all(vec![message]);
    let missing = v.checkout.missing();
    if !missing.is_empty() {
        use crate::i18n::t::checkout::missing as m;
        use domain::checkout::Missing;
        let messages = missing
            .into_iter()
            .map(|field| {
                match field {
                    Missing::CompanyName => m::company_name(l),
                    Missing::DeliveryStreet => m::delivery_street(l),
                    Missing::DeliveryCity => m::delivery_city(l),
                    Missing::DeliveryZip => m::delivery_zip(l),
                    Missing::DeliveryCountry => m::delivery_country(l),
                }
                .to_owned()
            })
            .collect();
        return refuse_all(messages).await;
    }
    if last(&fields)("terms_agreement").as_deref() != Some("1") {
        return refuse(crate::i18n::t::checkout::terms_required(l).to_owned()).await;
    }
    let draft = crate::orders::draft(&state, &v).await;
    let id = match crate::orders::place(&state, &v, &draft).await {
        Ok(id) => id,
        Err(crate::orders::PlaceError::Refused(message)) => return refuse(message).await,
        Err(e) => {
            tracing::error!("order not saved: {e}");
            return refuse(crate::i18n::t::checkout::submit_error(l).to_owned()).await;
        }
    };
    if let Some(user) = &v.user
        && let Err(e) = crate::orders::save_to_account(&state, user.id, &v.checkout).await
    {
        tracing::warn!("details not kept on the account: {e}");
    }
    crate::orders::send_emails(&v, &draft, id);
    v.cart.clear();
    v.checkout.clear();
    let _ = crate::session::save_cart(&session, &v.cart).await;
    let _ = save_checkout(&session, &v.checkout).await;
    let _ = session.insert(LAST_ORDER, id).await;
    Redirect::to(&l.path("/kosik/dekujeme")).into_response()
}
