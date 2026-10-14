//! The customer's account: overview with order history, editing the
//! profile, ordering again, the profile picture. All of it needs a login.

mod avatar;
mod edit;
mod reorder;
mod show;

pub use avatar::{change as avatar_change, destroy as avatar_destroy, show as avatar, update as avatar_update};
pub use edit::{show as edit, update};
pub use reorder::reorder;
pub use show::show;

use axum::response::Response;
use minijinja::{Value, context};

use crate::accounts::current_user::CurrentUser;
use crate::request::Req;
use crate::session::Flash;
use crate::views::redirect;

/// The signed-in customer, or the redirect to the sign-in page (which comes
/// back here afterwards). Boxed: a response is big, the user small.
pub(super) fn require_login(req: &Req) -> Result<&CurrentUser, Box<Response>> {
    if let Some(user) = &req.user {
        return Ok(user);
    }
    let here = match &req.query {
        Some(query) => format!("{}?{query}", req.path),
        None => req.path.clone(),
    };
    req.session.update(|data| data.return_to = Some(here));
    req.session
        .set_flash(Flash::Alert("Pro zobrazení účtu se musíte přihlásit.".into()));
    Err(Box::new(redirect::to(&req.path_for("/prihlaseni"))))
}

/// The account as the templates show it.
pub(super) fn account_view(req: &Req, current: &CurrentUser) -> Value {
    let user = &current.user;
    context! {
        email => &user.email_address,
        initial => user.email_address.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default(),
        is_admin => user.is_admin(),
        since => user.created_at.format("%d.%m.%Y").to_string(),
        avatar_path => current.avatar_path(|path| req.path_for(path)),
        phone_prefix => user.phone_prefix(),
        phone_number => user.phone_number(),
        first_name => &user.first_name,
        last_name => &user.last_name,
        billing_street => &user.billing_street,
        billing_city => &user.billing_city,
        billing_zip => &user.billing_zip,
        billing_country => &user.billing_country,
        company_purchase => user.company_purchase,
        company_name => &user.company_name,
        company_ico => &user.company_ico,
        company_dic => &user.company_dic,
        delivery_address_different => user.delivery_address_different,
        delivery_street => &user.delivery_street,
        delivery_city => &user.delivery_city,
        delivery_zip => &user.delivery_zip,
        delivery_country => &user.delivery_country,
    }
}
