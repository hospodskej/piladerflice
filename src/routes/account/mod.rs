//! The customer's account: overview with order history, editing the
//! profile, ordering again, the profile picture. All of it needs a login.

mod avatar;
mod edit;
mod reorder;
mod show;

pub use avatar::{destroy as avatar_destroy, show as avatar, update as avatar_update};
pub use edit::{show as edit, update};
pub use reorder::reorder;
pub use show::show;

use axum::response::Response;

use crate::AppState;
use crate::accounts::current_user::CurrentUser;
use crate::request::Req;
use crate::session::Flash;
use crate::ui::account::AccountView;
use crate::views::redirect;

/// The signed-in customer, or the redirect to the sign-in page (which comes
/// back here afterwards). Boxed: a response is big, the user small.
pub(super) fn require_login<'a>(state: &AppState, req: &'a Req) -> Result<&'a CurrentUser, Box<Response>> {
    if let Some(user) = &req.user {
        return Ok(user);
    }
    let here = match &req.query {
        Some(query) => format!("{}?{query}", req.path),
        None => req.path.clone(),
    };
    req.session.update(|data| data.return_to = Some(here));
    req.session.set_flash(Flash::Alert(state.translations.text(
        req.locale,
        "auth.login_required",
        &[],
    )));
    Err(Box::new(redirect::to(&req.path_for("/prihlaseni"))))
}

/// The account as its pages show it.
pub(super) fn account_view(req: &Req, current: &CurrentUser) -> AccountView {
    let user = &current.user;
    AccountView {
        email: user.email_address.clone(),
        initial: user
            .email_address
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default(),
        is_admin: user.is_admin(),
        since: crate::czech_time::from_utc(user.created_at)
            .format("%d.%m.%Y")
            .to_string(),
        avatar_path: current.avatar_path(|path| req.path_for(path)),
        phone_prefix: user.phone_prefix().to_string(),
        phone_number: user.phone_number(),
        first_name: user.first_name.clone(),
        last_name: user.last_name.clone(),
        billing_street: user.billing_street.clone(),
        billing_city: user.billing_city.clone(),
        billing_zip: user.billing_zip.clone(),
        billing_country: user.billing_country.clone(),
        company_purchase: user.company_purchase,
        company_name: user.company_name.clone(),
        company_ico: user.company_ico.clone(),
        company_dic: user.company_dic.clone(),
        delivery_address_different: user.delivery_address_different,
        delivery_street: user.delivery_street.clone(),
        delivery_city: user.delivery_city.clone(),
        delivery_zip: user.delivery_zip.clone(),
        delivery_country: user.delivery_country.clone(),
    }
}
