//! /muj-ucet/upravit: the profile (name, phone, addresses, company), which
//! checkout fills itself from.

use axum::extract::State;
use minijinja::context;

use super::{account_view, require_login};
use crate::AppState;
use crate::forms::FormData;
use crate::models::user::{self, PHONE_PREFIXES, PROFILE_FIELDS};
use crate::request::Req;
use crate::routes::Result;
use crate::session::Flash;
use crate::views::{self, Page, redirect};

pub async fn show(State(state): State<AppState>, req: Req) -> Result {
    let current = match require_login(&req) {
        Ok(current) => current,
        Err(redirect) => return Ok(*redirect),
    };
    let account = account_view(&req, current);
    let page = Page::meta(&state, &req, "account_edit");
    Ok(views::render(&state, &req, "accounts/edit.html", page, context! { account }).await)
}

pub async fn update(State(state): State<AppState>, req: Req, form: FormData) -> Result {
    let current = match require_login(&req) {
        Ok(current) => current,
        Err(redirect) => return Ok(*redirect),
    };
    user::update_profile(&state.db, current.user.id, &profile_fields(&form)).await?;
    req.session.set_flash(Flash::Notice(state.translations.text(
        req.locale,
        "auth.profile_updated",
        &[],
    )));
    Ok(redirect::to(&req.path_for("/muj-ucet")))
}

/// The submitted profile: the fields that were sent, checkboxes as
/// "true"/"false", and the phone put together from prefix and number.
fn profile_fields(form: &FormData) -> Vec<(String, Option<String>)> {
    let mut fields: Vec<(String, Option<String>)> = PROFILE_FIELDS
        .iter()
        .filter(|name| **name != "phone")
        .filter_map(|name| {
            let value = form.get(&format!("user[{name}]"))?;
            let value = match *name {
                "company_purchase" | "delivery_address_different" => {
                    (!matches!(value.trim(), "" | "0" | "false" | "off")).to_string()
                }
                _ => value.to_string(),
            };
            Some((name.to_string(), Some(value)))
        })
        .collect();
    let prefix = form
        .get("user[phone_prefix]")
        .filter(|prefix| PHONE_PREFIXES.contains(prefix))
        .unwrap_or(PHONE_PREFIXES[0]);
    let number = form.get("user[phone_number]").unwrap_or_default().trim();
    fields.push((
        "phone".into(),
        (!number.is_empty()).then(|| format!("{prefix} {number}")),
    ));
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn puts_the_phone_together_and_reads_checkboxes() {
        let form = FormData::parse(
            "user%5Bphone_prefix%5D=%2B43&user%5Bphone_number%5D=+660+1&user%5Bcompany_purchase%5D=0&user%5Bcompany_purchase%5D=1&user%5Brole%5D=admin"
                .as_bytes(),
        );
        let fields = profile_fields(&form);
        assert!(fields.contains(&("phone".into(), Some("+43 660 1".into()))));
        assert!(fields.contains(&("company_purchase".into(), Some("true".into()))));
        assert!(!fields.iter().any(|(name, _)| name == "role"));
        let fields = profile_fields(&FormData::parse(b"user%5Bphone_prefix%5D=%2B1&user%5Bphone_number%5D="));
        assert!(fields.contains(&("phone".into(), None)));
    }
}
