//! Signing in and creating an account.

use leptos::prelude::*;

use super::{CsrfField, path, t, t_html_with, t_with};

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <AuthBox title_key="auth.login_title">
            <form class="auth-form" action=path("/prihlaseni") accept-charset="UTF-8" method="post">
                <CsrfField/>
                <div class="form-group">
                    <label for="email_address">{t("kontakt.form_email")}</label>
                    <input type="email" name="email_address" id="email_address" required=true autofocus=true/>
                </div>
                <div class="form-group">
                    <label for="password">{t("auth.password")}</label>
                    <input type="password" name="password" id="password" required=true/>
                </div>
                <input type="submit" value=t("auth.login_submit") class="btn-primary auth-submit"/>
            </form>
            <p class="auth-switch">
                {t("auth.no_account")}
                " "
                <a href=path("/registrace")>{t("auth.register_link")}</a>
            </p>
        </AuthBox>
    }
}

/// `email` as typed before, `errors` why the account wasn't created.
#[component]
pub fn RegisterPage(email: String, errors: Vec<String>, error_count: String) -> impl IntoView {
    let privacy_link = format!(
        r#"<a target="_blank" rel="noopener" href="{}">{}</a>"#,
        path("/zasady-ochrany-osobnich-udaju"),
        sapwood_i18n::escape_html(&t("auth.privacy_link_text"))
    );
    let privacy_note =
        t_html_with("auth.register_privacy_html", &[("privacy_link", "\u{0}")]).replace('\u{0}', &privacy_link);
    view! {
        <AuthBox title_key="auth.register_title">
            {(!errors.is_empty())
                .then(|| {
                    view! {
                        <div class="page-flash page-flash-alert">
                            <strong>{t_with("auth.register_errors", &[("count", &error_count)])}</strong>
                            <ul>{errors.into_iter().map(|message| view! { <li>{message}</li> }).collect_view()}</ul>
                        </div>
                    }
                })}
            <form class="auth-form" action=path("/registrace") accept-charset="UTF-8" method="post">
                <CsrfField/>
                <div class="form-group">
                    <label for="email_address">{t("kontakt.form_email")}</label>
                    <input type="email" name="email_address" id="email_address" value=email required=true autofocus=true/>
                </div>
                <div class="form-group">
                    <label for="password">{t("auth.password")}</label>
                    <input type="password" name="password" id="password" required=true/>
                </div>
                <div class="form-group">
                    <label for="password_confirmation">{t("auth.password_confirmation")}</label>
                    <input type="password" name="password_confirmation" id="password_confirmation" required=true/>
                </div>
                <p class="auth-privacy-note" inner_html=privacy_note></p>
                <input type="submit" value=t("auth.register_submit") class="btn-primary auth-submit"/>
            </form>
            <p class="auth-switch">
                {t("auth.have_account")}
                " "
                <a href=path("/prihlaseni")>{t("auth.login_link")}</a>
            </p>
        </AuthBox>
    }
}

#[component]
fn AuthBox(title_key: &'static str, children: Children) -> impl IntoView {
    view! {
        <main class="main">
            <section class="auth-section">
                <div class="auth-box">
                    <h1 class="auth-box-title">{t(title_key)}</h1>
                    <div class="auth-box-body">{children()}</div>
                </div>
            </section>
        </main>
    }
}
