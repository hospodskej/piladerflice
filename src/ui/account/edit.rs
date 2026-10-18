//! /muj-ucet/upravit: the profile picture and the profile (name, phone,
//! addresses, company), which checkout fills itself from.

use leptos::prelude::*;

use super::{AccountView, Avatar};
use crate::islands::account::{AvatarTexts, AvatarUpload, PhonePrefix};
use crate::ui::{CsrfField, path, t};

#[component]
pub fn EditPage(account: AccountView) -> impl IntoView {
    let text = |name: &'static str, label: &'static str, value: &Option<String>, wide: bool| {
        view! {
            <div class="form-group" class:form-group-wide=wide>
                <label for=name>{t(label)}</label>
                <input type="text" name=name id=name value=value.clone().unwrap_or_default()/>
            </div>
        }
    };
    view! {
        <main class="main">
            <section class="account-dashboard">
                <div class="account-form-header">
                    <h1>{t("auth.edit_profile_title")}</h1>
                    " "
                    <a class="btn-outline-dark account-back-btn" href=path("/muj-ucet")>{t("auth.back_to_account")}</a>
                </div>
                <AvatarBox avatar_path=account.avatar_path.clone() initial=account.initial.clone()/>
                <form class="checkout-main-col" action=path("/muj-ucet/upravit") accept-charset="UTF-8" method="post">
                    <CsrfField/>
                    <div class="checkout-box">
                        <h3 class="checkout-box-title">{t("checkout.personal_details")}</h3>
                        <div class="checkout-box-body">
                            <div class="checkout-form-grid">
                                {text("first_name", "kontakt.form_jmeno", &account.first_name, false)}
                                {text("last_name", "kontakt.form_prijmeni", &account.last_name, false)}
                                <div class="form-group">
                                    <label for="phone_number">{t("kontakt.form_telefon")}</label>
                                    <div class="phone-field">
                                        <PhonePrefix prefix=account.phone_prefix.clone() label=t("auth.phone_prefix_label")/>
                                        <input type="tel" name="phone_number" id="phone_number" value=account.phone_number.clone()/>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="checkout-box">
                        <h3 class="checkout-box-title">{t("checkout.billing_details")}</h3>
                        <div class="checkout-box-body">
                            <div class="checkout-form-grid">
                                {text("billing_street", "checkout.street_and_number", &account.billing_street, true)}
                                {text("billing_city", "kontakt.form_mesto", &account.billing_city, false)}
                                {text("billing_zip", "kontakt.form_psc", &account.billing_zip, false)}
                                <Country name="billing_country" value=account.billing_country.clone()/>
                            </div>
                        </div>
                    </div>
                    <details class="checkout-box checkout-collapsible" open=account.company_purchase>
                        <summary class="checkout-box-title">{t("checkout.company_purchase")}</summary>
                        <div class="checkout-box-body">
                            <Checkbox name="company_purchase" checked=account.company_purchase/>
                            <div class="checkout-form-grid">
                                {text("company_name", "checkout.company_name", &account.company_name, true)}
                                {text("company_ico", "checkout.company_ico", &account.company_ico, false)}
                                {text("company_dic", "checkout.company_dic", &account.company_dic, false)}
                            </div>
                        </div>
                    </details>
                    <details class="checkout-box checkout-collapsible" open=account.delivery_address_different>
                        <summary class="checkout-box-title">{t("checkout.delivery_address_different")}</summary>
                        <div class="checkout-box-body">
                            <Checkbox name="delivery_address_different" checked=account.delivery_address_different/>
                            <div class="checkout-form-grid">
                                {text("delivery_street", "checkout.street_and_number", &account.delivery_street, true)}
                                {text("delivery_city", "kontakt.form_mesto", &account.delivery_city, false)}
                                {text("delivery_zip", "kontakt.form_psc", &account.delivery_zip, false)}
                                <Country name="delivery_country" value=account.delivery_country.clone()/>
                            </div>
                        </div>
                    </details>
                    <div class="checkout-nav">
                        <a class="btn-outline-dark" href=path("/muj-ucet")>{t("checkout.back")}</a>
                        " "
                        <input type="submit" value=t("auth.save_profile") class="btn-primary"/>
                    </div>
                </form>
            </section>
        </main>
    }
}

/// The profile picture: choose and upload one, or remove it.
#[component]
fn AvatarBox(avatar_path: Option<String>, initial: String) -> impl IntoView {
    let has_avatar = avatar_path.is_some();
    view! {
        <div class="checkout-box avatar-box">
            <h3 class="checkout-box-title">{t("auth.avatar_title")}</h3>
            <div class="checkout-box-body avatar-box-body">
                <Avatar avatar_path=avatar_path initial=initial extra_class="avatar-preview"/>
                <div class="avatar-controls">
                    <AvatarUpload texts=AvatarTexts {
                        action: path("/muj-ucet/profilovy-obrazek"),
                        choose: t("auth.avatar_choose"),
                        empty: t("auth.avatar_no_file"),
                        upload: t("auth.avatar_upload"),
                    }>
                        <CsrfField/>
                    </AvatarUpload>
                    {has_avatar
                        .then(|| {
                            view! {
                                <form class="button_to" method="post" action=path("/muj-ucet/profilovy-obrazek/smazat")>
                                    <button class="btn-outline-dark avatar-btn" type="submit">{t("auth.avatar_remove")}</button>
                                    <CsrfField/>
                                </form>
                            }
                        })}
                    <p class="avatar-hint">{t("auth.avatar_hint")}</p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Country(name: &'static str, value: Option<String>) -> impl IntoView {
    let value = value.filter(|value| !value.is_empty()).unwrap_or_else(|| "cz".into());
    view! {
        <div class="form-group">
            <label for=name>{t("checkout.country_field")}</label>
            <select name=name id=name>
                <option value="cz" selected=value == "cz">{t("checkout.country_cz")}</option>
                <option value="at" selected=value == "at">{t("checkout.country_at")}</option>
            </select>
        </div>
    }
}

#[component]
fn Checkbox(name: &'static str, checked: bool) -> impl IntoView {
    view! {
        <label class="custom-checkbox-label checkout-checkbox">
            <input type="checkbox" name=name value="1" checked=checked/>
            <span class="custom-checkbox"></span>
            " "
            <span>{t(&format!("checkout.{name}"))}</span>
        </label>
    }
}
