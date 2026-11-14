//! Signing in, registering, the forgotten password and the customer's account.

use super::{PageMeta, loaded};
use crate::i18n::t;
use crate::layout::{FlagAt, FlagCz, Layout};
use crate::visitor::{Visitor, visitor};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Private pages aren't cached anywhere.
fn no_store() {
    #[cfg(feature = "ssr")]
    if let Some(response) = use_context::<leptos_axum::ResponseOptions>() {
        response.insert_header(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-store"),
        );
    }
}

#[allow(unused_variables)]
fn redirect(v: &Visitor, path: &str) {
    #[cfg(feature = "ssr")]
    leptos_axum::redirect(&v.link(path));
}

/// Remembers the sign-in page's `return_to` (a page of this site) for after signing in.
#[server(endpoint = "page/login")]
pub async fn remember_return_to() -> Result<(), ServerFnError> {
    let session: tower_sessions::Session = leptos_axum::extract().await?;
    let uri: axum::http::Uri = leptos_axum::extract().await?;
    crate::forms::auth::remember_return_to(&session, &uri).await;
    Ok(())
}

fn auth_box(title: &'static str, body: impl IntoView) -> impl IntoView {
    view! {
        <main class="main">
            <section class="auth-section">
                <div class="auth-box">
                    <h1 class="auth-box-title">{title}</h1>
                    <div class="auth-box-body">{body}</div>
                </div>
            </section>
        </main>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    no_store();
    let v = visitor();
    let l = v.locale;
    let body = view! {
        {loaded(remember_return_to(), |()| ())}
        <form action={v.link("/prihlaseni")} method="post" class="auth-form">
            <div class="form-group">
                <label for="email_address">{t::kontakt::form_email(l)}</label>
                <input type="email" name="email_address" id="email_address" required autofocus/>
            </div>
            <div class="form-group">
                <label for="password">{t::auth::password(l)}</label>
                <input type="password" name="password" id="password" required/>
            </div>
            <input type="submit" value=t::auth::login_submit(l) class="btn-primary auth-submit"/>
        </form>
        <p class="auth-forgot"><a href=v.link("/zapomenute-heslo")>{t::auth::reset::forgot_link(l)}</a></p>
        <p class="auth-switch">{t::auth::no_account(l)}" "<a href=v.link("/registrace")>{t::auth::register_link(l)}</a></p>
    };
    view! {
        <PageMeta title=t::meta::login::title(l).to_owned()/>
        <Layout>{auth_box(t::auth::login_title(l), body)}</Layout>
    }
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    no_store();
    let v = visitor();
    let l = v.locale;
    let (email, problems) = v.registration_form.clone().unwrap_or_default();
    let privacy = codon::Html::from_trusted(format!(
        r#"<a target="_blank" rel="noopener" href="{}">{}</a>"#,
        v.link("/zasady-ochrany-osobnich-udaju"),
        codon::Html::escape(t::auth::privacy_link_text(l))
    ));
    let count = i64::try_from(problems.len()).unwrap_or(0);
    let body = view! {
        {(!problems.is_empty()).then(|| view! {
            <div class="page-flash page-flash-alert">
                <strong>{t::auth::errors::prevent_registration(l, count)}</strong>
                <ul>{problems.into_iter().map(|p| view! { <li>{p}</li> }).collect_view()}</ul>
            </div>
        })}
        <form action={v.link("/registrace")} method="post" class="auth-form">
            <div class="form-group">
                <label for="user_email_address">{t::kontakt::form_email(l)}</label>
                <input type="email" name="user[email_address]" id="user_email_address" value=email required autofocus/>
            </div>
            <div class="form-group">
                <label for="user_password">{t::auth::password(l)}</label>
                <input type="password" name="user[password]" id="user_password" required/>
            </div>
            <div class="form-group">
                <label for="user_password_confirmation">{t::auth::password_confirmation(l)}</label>
                <input type="password" name="user[password_confirmation]" id="user_password_confirmation" required/>
            </div>
            <p class="auth-privacy-note" inner_html=t::auth::register_privacy(l, &privacy).into_string()></p>
            <input type="submit" value=t::auth::register_submit(l) class="btn-primary auth-submit"/>
        </form>
        <p class="auth-switch">{t::auth::have_account(l)}" "<a href=v.link("/prihlaseni")>{t::auth::login_link(l)}</a></p>
    };
    view! {
        <PageMeta title=t::meta::register::title(l).to_owned()/>
        <Layout>{auth_box(t::auth::register_title(l), body)}</Layout>
    }
}

#[component]
pub fn ForgotPasswordPage() -> impl IntoView {
    no_store();
    let v = visitor();
    let l = v.locale;
    use t::auth::reset as r;
    let body = view! {
        <p class="auth-privacy-note">{r::forgot_intro(l)}</p>
        <form action={v.link("/zapomenute-heslo")} method="post" class="auth-form">
            <div class="form-group">
                <label for="email_address">{t::kontakt::form_email(l)}</label>
                <input type="email" name="email_address" id="email_address" required autofocus autocomplete="email"/>
            </div>
            <input type="submit" value=r::forgot_submit(l) class="btn-primary auth-submit"/>
        </form>
        <p class="auth-switch"><a href=v.link("/prihlaseni")>{r::back_to_login(l)}</a></p>
    };
    view! {
        <PageMeta title=t::meta::forgot_password::title(l).to_owned()/>
        <Layout>{auth_box(r::forgot_title(l), body)}</Layout>
    }
}

/// The e-mail the reset link is for, or `None` when the link is no good.
#[server(endpoint = "page/reset")]
pub async fn load_reset() -> Result<Option<String>, ServerFnError> {
    let session: tower_sessions::Session = leptos_axum::extract().await?;
    let state = expect_context::<crate::state::AppState>();
    let Some(token) = crate::forms::auth::reset_token(&session).await else {
        return Ok(None);
    };
    Ok(crate::forms::auth::reset_user(&state, &token)
        .await
        .map(|(_, email)| email))
}

#[component]
pub fn ResetPasswordPage() -> impl IntoView {
    no_store();
    let v = visitor();
    let l = v.locale;
    use t::auth::reset as r;
    let body = loaded(load_reset(), move |email: Option<String>| {
        let v = visitor();
        match email {
            None => {
                redirect(&v, "/zapomenute-heslo");
                ().into_any()
            }
            Some(email) => view! {
                <p class="auth-privacy-note">{r::new_intro(l, &email)}</p>
                <form action={v.link("/nove-heslo")} method="post" class="auth-form">
                    <div class="form-group">
                        <label for="password">{r::new_password(l)}</label>
                        <input type="password" name="password" id="password" required minlength="8" autocomplete="new-password" autofocus/>
                    </div>
                    <div class="form-group">
                        <label for="password_confirmation">{t::auth::password_confirmation(l)}</label>
                        <input type="password" name="password_confirmation" id="password_confirmation" required minlength="8" autocomplete="new-password"/>
                    </div>
                    <input type="submit" value=r::new_submit(l) class="btn-primary auth-submit"/>
                </form>
            }
            .into_any(),
        }
    });
    view! {
        <PageMeta title=t::meta::reset_password::title(l).to_owned()/>
        <Layout>{auth_box(r::new_title(l), body)}</Layout>
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderCard {
    id: i64,
    date: String,
    items: String,
    total: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountContent {
    since: String,
    orders: Vec<OrderCard>,
}

#[server(endpoint = "page/account")]
pub async fn load_account() -> Result<Option<AccountContent>, ServerFnError> {
    let v = visitor();
    let Some(user) = &v.user else { return Ok(None) };
    let db = &expect_context::<crate::state::AppState>().db;
    let err = |e: sqlx::Error| ServerFnError::new(e.to_string());
    let since = sqlx::query_scalar!("SELECT created_at FROM users WHERE id = ?", user.id)
        .fetch_one(db)
        .await
        .map_err(err)?;
    let date = |iso: &str| {
        let date = iso.get(..10).unwrap_or_default();
        let mut parts = date.split('-');
        let (y, m, d) = (
            parts.next().unwrap_or_default(),
            parts.next().unwrap_or_default(),
            parts.next().unwrap_or_default(),
        );
        format!("{d}.{m}.{y}")
    };
    let orders = sqlx::query!("SELECT id, created_at, items, total_czk FROM orders WHERE user_id = ? ORDER BY created_at DESC LIMIT 10", user.id)
        .fetch_all(db)
        .await
        .map_err(err)?
        .into_iter()
        .map(|o| {
            let count = serde_json::from_str::<Vec<serde_json::Value>>(&o.items).map_or(0, |items| items.len());
            OrderCard {
                id: o.id,
                date: date(&o.created_at),
                items: t::auth::order_item_count(v.locale, i64::try_from(count).unwrap_or(0)),
                total: v.shop.price(v.locale, domain::Czk(o.total_czk)),
            }
        })
        .collect();
    Ok(Some(AccountContent {
        since: date(&since),
        orders,
    }))
}

/// The profile picture, or the e-mail's first letter.
fn avatar(v: &Visitor) -> AnyView {
    let user = v.user.as_ref();
    match user.and_then(|u| u.avatar.clone()) {
        Some(src) => {
            view! { <img src=src alt=t::auth::avatar_alt(v.locale) width="64" height="64"/> }
                .into_any()
        }
        None => user
            .and_then(|u| u.email.chars().next())
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default()
            .into_any(),
    }
}

#[component]
pub fn AccountPage() -> impl IntoView {
    no_store();
    let v = visitor();
    let l = v.locale;
    if v.user.is_none() {
        redirect(&v, "/prihlaseni");
    }
    let body = loaded(load_account(), move |content: Option<AccountContent>| {
        let v = visitor();
        let Some(content) = content else {
            return ().into_any();
        };
        let Some(user) = v.user.clone() else {
            return ().into_any();
        };
        let count = content.orders.len();
        view! {
            <main class="main">
                <section class="account-dashboard">
                    <header class="account-profile-card">
                        <div class="account-avatar">{avatar(&v)}</div>
                        <div class="account-profile-info">
                            <h1 class="account-profile-title">
                                {t::auth::account_title(l)}
                                {user.admin.then(|| view! { " "<span class="account-admin-badge">{t::auth::admin_badge(l)}</span> })}
                            </h1>
                            <p class="account-profile-email">{user.email.clone()}</p>
                            <p class="account-profile-since">{t::auth::customer_since(l, &content.since)}</p>
                        </div>
                        <div class="account-profile-actions">
                            {user.admin.then(|| view! { <a href="/admin" class="btn-primary account-action-btn">{t::auth::go_to_admin(l)}</a> })}
                            <a href=v.link("/muj-ucet/upravit") class="btn-outline-dark account-action-btn">{t::auth::edit_profile_link(l)}</a>
                            <form class="button_to" method="post" action={v.link("/odhlaseni")}>
                                <button class="btn-outline-dark account-action-btn" type="submit">{t::auth::logout(l)}</button>
                            </form>
                        </div>
                    </header>
                    <section class="account-orders-section">
                        <div class="account-orders-heading">
                            <h2>{t::auth::order_history_title(l)}</h2>
                            {(count > 0).then(|| view! { <span class="account-orders-count">{count}</span> })}
                        </div>
                        {if count > 0 {
                            view! {
                                <div class="order-card-list">
                                    {content.orders.into_iter().map(|o| view! {
                                        <div class="order-card">
                                            <div class="order-card-main">
                                                <span class="order-card-number">{t::auth::order_number(l)}" "{o.id}</span>
                                                <span class="order-card-meta">{o.date}" · "{o.items}</span>
                                            </div>
                                            <span class="order-card-total">{o.total}</span>
                                            <form class="button_to" method="post" action={v.link(&format!("/muj-ucet/objednavky/{}/znovu-objednat", o.id))}>
                                                <button class="btn-outline-dark order-card-reorder-btn" type="submit">{t::auth::reorder_button(l)}</button>
                                            </form>
                                        </div>
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="account-orders-empty">
                                    <p>{t::auth::no_orders(l)}</p>
                                    <a href=v.link("/eshop") class="btn-primary">{t::nav::eshop(l)}</a>
                                </div>
                            }.into_any()
                        }}
                    </section>
                </section>
            </main>
        }
        .into_any()
    });
    view! {
        <PageMeta title=t::meta::account::title(l).to_owned()/>
        <Layout>{body}</Layout>
    }
}

/// The styled file input: shows the chosen file's name; upload waits for a file.
#[island]
fn FilePicker(choose: String, empty: String, upload: String) -> impl IntoView {
    let name = RwSignal::new(None::<String>);
    view! {
        <input
            type="file"
            name="avatar"
            id="avatar_file"
            class="file-picker-input"
            accept="image/jpeg,image/png,image/webp"
            required
            on:change=move |e| {
                #[cfg(feature = "hydrate")]
                {
                    use wasm_bindgen::JsCast;
                    let file = e.target()
                        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                        .and_then(|i| i.files())
                        .and_then(|f| f.get(0));
                    name.set(file.map(|f| f.name()));
                }
                let _ = e;
            }
        />
        <label for="avatar_file" class="btn-outline-dark file-picker-btn">{choose}</label>
        <span class="file-picker-name" class:has-file=move || name.with(Option::is_some)>{move || name.get().unwrap_or_else(|| empty.clone())}</span>
        <input type="submit" value=upload class="btn-primary avatar-btn" disabled=move || name.with(Option::is_none)/>
    }
}

/// The phone prefix (+420 / +43) beside the number.
#[island]
fn PhonePrefix(prefix: String, label: String) -> impl IntoView {
    let chosen = RwSignal::new(prefix);
    let open = RwSignal::new(false);
    let button = NodeRef::<leptos::html::Button>::new();
    let menu_style = RwSignal::new(String::new());
    let root = NodeRef::<leptos::html::Div>::new();
    #[cfg(feature = "hydrate")]
    let _ = window_event_listener(leptos::ev::click, move |e| {
        use wasm_bindgen::JsCast;
        let node = e.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok());
        if !root
            .get_untracked()
            .is_some_and(|r| r.contains(node.as_ref()))
        {
            open.set(false);
        }
    });
    let flag = move |p: &str| {
        if p == "+43" {
            view! { <FlagAt/> }.into_any()
        } else {
            view! { <FlagCz/> }.into_any()
        }
    };
    let option = move |value: &'static str| {
        view! {
            <div class="phone-prefix-option" data-value=value on:click=move |e| { e.stop_propagation(); chosen.set(value.to_owned()); open.set(false); }>
                {flag(value)}<span>{value}</span>
            </div>
        }
    };
    view! {
        <div class="phone-prefix-dropdown" node_ref=root>
            <button
                type="button"
                class="phone-prefix-btn"
                class:open=open
                aria-label=label
                node_ref=button
                on:click=move |e| {
                    e.stop_propagation();
                    if !open.get_untracked() {
                        // The menu is placed under the button, above everything.
                        if let Some(b) = button.get_untracked() {
                            let r = b.get_bounding_client_rect();
                            menu_style.set(format!("position: fixed; top: {}px; left: {}px; min-width: {}px", r.bottom() + 4.0, r.left(), r.width()));
                        }
                    }
                    open.update(|o| *o = !*o);
                }
            >
                <span>{move || flag(&chosen.get())}</span>
                <span>{move || chosen.get()}</span>
                <span class="chevron">"▼"</span>
            </button>
            <input type="hidden" name="user[phone_prefix]" prop:value=chosen/>
            <div class="phone-prefix-menu" class:open=open style=menu_style>
                {option("+420")}
                {option("+43")}
            </div>
        </div>
    }
}

#[component]
pub fn AccountEditPage() -> impl IntoView {
    no_store();
    let v = visitor();
    let l = v.locale;
    let Some(user) = v.user.clone() else {
        redirect(&v, "/prihlaseni");
        return ().into_any();
    };
    let p = user.profile.clone();
    let val = |o: &Option<String>| o.clone().unwrap_or_default();
    let phone = p.phone.clone().unwrap_or_default();
    let prefix = ["+420", "+43"]
        .into_iter()
        .find(|x| phone.starts_with(x))
        .unwrap_or("+420");
    let number = phone
        .strip_prefix(prefix)
        .unwrap_or(&phone)
        .trim()
        .to_owned();
    let field = move |name: &'static str, label: &'static str, value: String, wide: bool| {
        view! {
            <div class=if wide { "form-group form-group-wide" } else { "form-group" }>
                <label for=format!("user_{name}")>{label}</label>
                <input type="text" name=format!("user[{name}]") id=format!("user_{name}") value=value/>
            </div>
        }
    };
    let country = move |name: &'static str, value: Option<String>| {
        let at = value.as_deref() == Some("at");
        view! {
            <div class="form-group">
                <label for=format!("user_{name}")>{t::checkout::country_field(l)}</label>
                <select name=format!("user[{name}]") id=format!("user_{name}")>
                    <option value="cz" selected=!at>{t::checkout::country_cz(l)}</option>
                    <option value="at" selected=at>{t::checkout::country_at(l)}</option>
                </select>
            </div>
        }
    };
    let checkbox = move |name: &'static str, checked: bool, label: &'static str| {
        view! {
            <label class="custom-checkbox-label checkout-checkbox">
                <input type="hidden" name=format!("user[{name}]") value="0"/>
                <input type="checkbox" name=format!("user[{name}]") value="1" checked=checked/>
                <span class="custom-checkbox"></span>
                <span>{label}</span>
            </label>
        }
    };
    let body = view! {
        <main class="main">
            <section class="account-dashboard">
                <div class="account-form-header">
                    <h1>{t::auth::edit_profile_title(l)}</h1>
                    <a href=v.link("/muj-ucet") class="btn-outline-dark account-back-btn">{t::auth::back_to_account(l)}</a>
                </div>
                <div class="checkout-box avatar-box">
                    <h3 class="checkout-box-title">{t::auth::avatar_title(l)}</h3>
                    <div class="checkout-box-body avatar-box-body">
                        <div class="account-avatar avatar-preview">{avatar(&v)}</div>
                        <div class="avatar-controls">
                            <form action={v.link("/muj-ucet/profilovy-obrazek")} method="post" enctype="multipart/form-data" class="avatar-upload-form">
                                <FilePicker choose=t::auth::avatar_choose(l).into() empty=t::auth::avatar_no_file(l).into() upload=t::auth::avatar_upload(l).into()/>
                            </form>
                            {user.avatar.is_some().then(|| view! {
                                <form class="button_to" method="post" action={v.link("/muj-ucet/profilovy-obrazek/smazat")}>
                                    <button class="btn-outline-dark avatar-btn" type="submit">{t::auth::avatar_remove(l)}</button>
                                </form>
                            })}
                            <p class="avatar-hint">{t::auth::avatar_hint(l)}</p>
                        </div>
                    </div>
                </div>
                <form action={v.link("/muj-ucet/upravit")} method="post" class="checkout-main-col">
                    <div class="checkout-box">
                        <h3 class="checkout-box-title">{t::checkout::personal_details(l)}</h3>
                        <div class="checkout-box-body">
                            <div class="checkout-form-grid">
                                {field("first_name", t::kontakt::form_jmeno(l), val(&p.first_name), false)}
                                {field("last_name", t::kontakt::form_prijmeni(l), val(&p.last_name), false)}
                                <div class="form-group">
                                    <label for="user_phone_number">{t::kontakt::form_telefon(l)}</label>
                                    <div class="phone-field">
                                        <PhonePrefix prefix=prefix.to_owned() label=t::auth::phone_prefix_label(l).to_owned()/>
                                        <input type="tel" name="user[phone_number]" id="user_phone_number" value=number/>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="checkout-box">
                        <h3 class="checkout-box-title">{t::checkout::billing_details(l)}</h3>
                        <div class="checkout-box-body">
                            <div class="checkout-form-grid">
                                {field("billing_street", t::checkout::street_and_number(l), val(&p.billing.street), true)}
                                {field("billing_city", t::kontakt::form_mesto(l), val(&p.billing.city), false)}
                                {field("billing_zip", t::kontakt::form_psc(l), val(&p.billing.zip), false)}
                                {country("billing_country", p.billing.country.clone())}
                            </div>
                        </div>
                    </div>
                    <details class="checkout-box checkout-collapsible" open=p.company_purchase>
                        <summary class="checkout-box-title">{t::checkout::company_purchase(l)}</summary>
                        <div class="checkout-box-body">
                            {checkbox("company_purchase", p.company_purchase, t::checkout::company_purchase(l))}
                            <div class="checkout-form-grid">
                                {field("company_name", t::checkout::company_name(l), val(&p.company_name), true)}
                                {field("company_ico", t::checkout::company_ico(l), val(&p.company_ico), false)}
                                {field("company_dic", t::checkout::company_dic(l), val(&p.company_dic), false)}
                            </div>
                        </div>
                    </details>
                    <details class="checkout-box checkout-collapsible" open=p.delivery_address_different>
                        <summary class="checkout-box-title">{t::checkout::delivery_address_different(l)}</summary>
                        <div class="checkout-box-body">
                            {checkbox("delivery_address_different", p.delivery_address_different, t::checkout::delivery_address_different(l))}
                            <div class="checkout-form-grid">
                                {field("delivery_street", t::checkout::street_and_number(l), val(&p.delivery.street), true)}
                                {field("delivery_city", t::kontakt::form_mesto(l), val(&p.delivery.city), false)}
                                {field("delivery_zip", t::kontakt::form_psc(l), val(&p.delivery.zip), false)}
                                {country("delivery_country", p.delivery.country.clone())}
                            </div>
                        </div>
                    </details>
                    <div class="checkout-nav">
                        <a href=v.link("/muj-ucet") class="btn-outline-dark">{t::checkout::back(l)}</a>
                        <input type="submit" value=t::auth::save_profile(l) class="btn-primary"/>
                    </div>
                </form>
            </section>
        </main>
    };
    view! {
        <PageMeta title=t::meta::account_edit::title(l).to_owned()/>
        <Layout>{body}</Layout>
    }
    .into_any()
}
