//! Step 3: name, e-mail, phone, billing address, and optionally a company
//! and a different delivery address.

use leptos::prelude::*;

use super::{CartSummary, CountrySelect, Stepper, value};
use crate::shop::checkout::CheckoutState;
use crate::ui::{CsrfField, path, t, t_with};

/// `user_email`: the signed-in customer; otherwise a link to sign in
/// (`login_link`, coming back here).
#[component]
pub fn DetailsPage(checkout: CheckoutState, user_email: Option<String>, login_link: String) -> impl IntoView {
    let field = |name: &'static str, label: &'static str, kind: &'static str, wide: bool, required: bool| {
        view! {
            <div class="form-group" class:form-group-wide=wide>
                <label for=name>{t(label)}</label>
                <input type=kind name=name id=name value=value(&checkout, name) required=required/>
            </div>
        }
    };
    let billing_country = checkout
        .get("billing_country")
        .or(checkout.get("country"))
        .unwrap_or("cz")
        .to_string();
    let delivery_country = checkout.get("delivery_country").unwrap_or("cz").to_string();
    view! {
        <main class="main">
            <section class="checkout-section">
                <Stepper current=3/>
                <div class="checkout-layout">
                    <form class="checkout-main-col" action=path("/kosik/udaje") accept-charset="UTF-8" method="post">
                        <CsrfField/>
                        <div class="checkout-box">
                            <h3 class="checkout-box-title">{t("checkout.personal_details")}</h3>
                            <div class="checkout-box-body">
                                {match user_email {
                                    Some(email) => {
                                        view! {
                                            <p class="checkout-logged-in-as">{t_with("checkout.logged_in_as", &[("email", &email)])}</p>
                                        }
                                            .into_any()
                                    }
                                    None => {
                                        view! {
                                            <a class="btn-outline-dark checkout-login-btn" href=login_link>
                                                {t("checkout.login_existing_user")}
                                            </a>
                                        }
                                            .into_any()
                                    }
                                }}
                                <div class="checkout-form-grid">
                                    {field("first_name", "kontakt.form_jmeno", "text", false, true)}
                                    {field("last_name", "kontakt.form_prijmeni", "text", false, true)}
                                    {field("email", "kontakt.form_email", "email", false, true)}
                                    {field("phone", "kontakt.form_telefon", "tel", false, true)}
                                </div>
                                <Checkbox name="newsletter_opt_in" checked=checkout.flag("newsletter_opt_in") label=t("checkout.newsletter_opt_in")/>
                            </div>
                        </div>
                        <div class="checkout-box">
                            <h3 class="checkout-box-title">{t("checkout.billing_details")}</h3>
                            <div class="checkout-box-body">
                                <div class="checkout-form-grid">
                                    {field("billing_street", "checkout.street_and_number", "text", true, true)}
                                    {field("billing_city", "kontakt.form_mesto", "text", false, true)}
                                    {field("billing_zip", "kontakt.form_psc", "text", false, true)}
                                    <div class="form-group">
                                        <label for="billing_country">{t("checkout.country_field")}</label>
                                        <CountrySelect name="billing_country" selected=billing_country/>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <details class="checkout-box checkout-collapsible" open=checkout.flag("company_purchase")>
                            <summary class="checkout-box-title">{t("checkout.company_purchase")}</summary>
                            <div class="checkout-box-body">
                                <Checkbox name="company_purchase" checked=checkout.flag("company_purchase") label=t("checkout.company_purchase")/>
                                <div class="checkout-form-grid">
                                    {field("company_name", "checkout.company_name", "text", true, false)}
                                    {field("company_ico", "checkout.company_ico", "text", false, false)}
                                    {field("company_dic", "checkout.company_dic", "text", false, false)}
                                </div>
                            </div>
                        </details>
                        <details class="checkout-box checkout-collapsible" open=checkout.flag("delivery_address_different")>
                            <summary class="checkout-box-title">{t("checkout.delivery_address_different")}</summary>
                            <div class="checkout-box-body">
                                <Checkbox
                                    name="delivery_address_different"
                                    checked=checkout.flag("delivery_address_different")
                                    label=t("checkout.delivery_address_different")
                                />
                                <div class="checkout-form-grid">
                                    {field("delivery_street", "checkout.street_and_number", "text", true, false)}
                                    {field("delivery_city", "kontakt.form_mesto", "text", false, false)}
                                    {field("delivery_zip", "kontakt.form_psc", "text", false, false)}
                                    <div class="form-group">
                                        <label for="delivery_country">{t("checkout.country_field")}</label>
                                        <CountrySelect name="delivery_country" selected=delivery_country/>
                                    </div>
                                </div>
                            </div>
                        </details>
                        <div class="checkout-nav">
                            <a class="btn-outline-dark" href=path("/kosik/doprava")>{t("checkout.back")}</a>
                            " "
                            <input type="submit" value=t("checkout.continue") class="btn-primary"/>
                        </div>
                    </form>
                    <CartSummary/>
                </div>
            </section>
        </main>
    }
}

/// A styled checkbox with its label.
#[component]
fn Checkbox(name: &'static str, checked: bool, label: String) -> impl IntoView {
    view! {
        <label class="custom-checkbox-label checkout-checkbox">
            <input type="checkbox" name=name value="1" checked=checked/>
            <span class="custom-checkbox"></span>
            " "
            <span>{label}</span>
        </label>
    }
}
