//! The customer's account: overview with orders, and the profile form.

mod edit;
mod show;

pub use edit::EditPage;
pub use show::{AccountPage, OrderCard};

use leptos::prelude::*;

use super::t;

/// The signed-in customer, as the account pages show them.
pub struct AccountView {
    pub email: String,
    /// The e-mail's first letter, when there is no picture.
    pub initial: String,
    pub is_admin: bool,
    /// "01.03.2025"
    pub since: String,
    pub avatar_path: Option<String>,
    pub phone_prefix: String,
    pub phone_number: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub billing_street: Option<String>,
    pub billing_city: Option<String>,
    pub billing_zip: Option<String>,
    pub billing_country: Option<String>,
    pub company_purchase: bool,
    pub company_name: Option<String>,
    pub company_ico: Option<String>,
    pub company_dic: Option<String>,
    pub delivery_address_different: bool,
    pub delivery_street: Option<String>,
    pub delivery_city: Option<String>,
    pub delivery_zip: Option<String>,
    pub delivery_country: Option<String>,
}

/// The round picture of the account (or the e-mail's first letter).
#[component]
fn Avatar(avatar_path: Option<String>, initial: String, #[prop(optional)] extra_class: &'static str) -> impl IntoView {
    let class = if extra_class.is_empty() {
        "account-avatar".to_string()
    } else {
        format!("account-avatar {extra_class}")
    };
    view! {
        <div class=class>
            {match avatar_path {
                Some(src) => view! { <img alt=t("auth.avatar_alt") width="64" height="64" src=src/> }.into_any(),
                None => view! { {initial} }.into_any(),
            }}
        </div>
    }
}
