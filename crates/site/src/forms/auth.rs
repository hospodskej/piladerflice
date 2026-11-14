//! Signing in and out, registering, the forgotten password, the account's
//! details, profile picture and "order again".

use super::{Fields, field, locale_of, redirect_with, too_many_attempts};
use crate::i18n::{Locale, t};
use crate::session::{self, RETURN_TO};
use crate::state::AppState;
use crate::visitor::{Flash, Visitor};
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Form};
use std::time::SystemTime;
use thymus::{Password, PasswordHash, normalize_email};
use tower_sessions::Session;

/// Shown again on the registration form after a refusal.
pub const REGISTRATION_FORM: &str = "registration_form";
/// The token from the e-mailed link, kept in the session (so it leaves the address bar).
const RESET_TOKEN: &str = "password_reset_token";

/// Hashing and checking passwords is slow on purpose; it runs off the async threads.
async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    tokio::task::spawn_blocking(f)
        .await
        .expect("password task panicked")
}

/// The sign-in form's `return_to`, kept for after signing in when it's a page of this site.
pub async fn remember_return_to(session: &Session, uri: &Uri) {
    let return_to = uri
        .query()
        .and_then(|q| q.split('&').find_map(|p| p.strip_prefix("return_to=")));
    if let Some(path) = return_to
        .map(|p| p.replace("%2F", "/"))
        .filter(|p| thymus::is_local_path(p))
    {
        let _ = session.insert(RETURN_TO, path).await;
    }
}

pub async fn sign_in(
    State(state): State<AppState>,
    session: Session,
    ip: refractory::RemoteIp,
    uri: Uri,
    Form(fields): Form<Fields>,
) -> Response {
    let l = locale_of(&uri);
    if let Some(refused) = too_many_attempts(
        &state,
        &session,
        &ip,
        &crate::limits::LOGIN,
        l,
        "/prihlaseni",
    )
    .await
    {
        return refused;
    }
    let email = normalize_email(field(&fields, "email_address").unwrap_or_default());
    // Also limited per account, from any address.
    if state
        .limiter
        .check(&crate::limits::LOGIN_PER_ACCOUNT, &email)
        .await
        .is_err()
    {
        return redirect_with(
            &session,
            l,
            "/prihlaseni",
            Some(Flash::Alert(t::security::too_many_attempts(l).into())),
        )
        .await;
    }
    let password = field(&fields, "password").unwrap_or_default().to_owned();
    let user = sqlx::query!(
        r#"SELECT id AS "id!", password_hash, session_epoch FROM users WHERE email_address = ?"#,
        email
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();
    let hash = user
        .as_ref()
        .and_then(|u| u.password_hash.parse::<PasswordHash>().ok());
    // The same work whether or not the e-mail has an account.
    let ok = blocking(move || thymus::authenticate(hash.as_ref(), &password)).await;
    match (ok, user) {
        (true, Some(user)) => {
            if let Err(e) = session::sign_in(&session, user.id, user.session_epoch).await {
                tracing::error!("not signed in: {e}");
            }
            let back: Option<String> = session.remove(RETURN_TO).await.ok().flatten();
            Redirect::to(&back.unwrap_or_else(|| l.path("/muj-ucet"))).into_response()
        }
        _ => {
            redirect_with(
                &session,
                l,
                "/prihlaseni",
                Some(Flash::Alert(t::auth::login_failed(l).into())),
            )
            .await
        }
    }
}

pub async fn sign_out(session: Session, uri: Uri) -> Response {
    let l = locale_of(&uri);
    if let Err(e) = session::sign_out(&session).await {
        tracing::warn!("sign out: {e}");
    }
    redirect_with(
        &session,
        l,
        "/",
        Some(Flash::Notice(t::auth::logged_out(l).into())),
    )
    .await
}

/// Why a password can't be used, for the forms (every reason, as before).
fn password_problems(l: Locale, password: &str, confirmation: Option<&str>) -> Vec<String> {
    use t::auth::errors as e;
    let mut problems = Vec::new();
    if password.is_empty() {
        problems.push(e::password_blank(l).into());
    } else if Password::new(password.to_owned()).is_err() {
        problems.push(e::password_too_short(l, Password::MIN_CHARS));
    }
    if confirmation.is_some_and(|c| c != password) {
        problems.push(e::password_confirmation(l).into());
    }
    problems
}

pub async fn register(
    State(state): State<AppState>,
    session: Session,
    ip: refractory::RemoteIp,
    uri: Uri,
    Form(fields): Form<Fields>,
) -> Response {
    use t::auth::errors as e;
    let l = locale_of(&uri);
    if let Some(refused) = too_many_attempts(
        &state,
        &session,
        &ip,
        &crate::limits::REGISTRATION,
        l,
        "/registrace",
    )
    .await
    {
        return refused;
    }
    let email = normalize_email(field(&fields, "user[email_address]").unwrap_or_default());
    let password = fields
        .iter()
        .find(|(k, _)| k == "user[password]")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let confirmation = fields
        .iter()
        .find(|(k, _)| k == "user[password_confirmation]")
        .map(|(_, v)| v.as_str());
    let mut problems = Vec::new();
    if email.is_empty() {
        problems.push(e::email_blank(l).to_owned());
    } else if sqlx::query_scalar!(
        "SELECT 1 AS \"one!: i64\" FROM users WHERE email_address = ?",
        email
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .is_some()
    {
        problems.push(e::email_taken(l).to_owned());
    }
    problems.extend(password_problems(l, &password, confirmation));
    if !problems.is_empty() {
        let _ = session.insert(REGISTRATION_FORM, (email, problems)).await;
        return Redirect::to(&l.path("/registrace")).into_response();
    }
    let Ok(password) = Password::new(password) else {
        unreachable!("checked above")
    };
    let Ok(hash) = blocking(move || PasswordHash::new(&password)).await else {
        return redirect_with(
            &session,
            l,
            "/registrace",
            Some(Flash::Alert(t::checkout::submit_error(l).into())),
        )
        .await;
    };
    let hash = hash.into_string();
    match sqlx::query_scalar!(
        r#"INSERT INTO users (email_address, password_hash) VALUES (?, ?) RETURNING id AS "id!""#,
        email,
        hash
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(id) => {
            let _ = session::sign_in(&session, id, 0).await;
            redirect_with(
                &session,
                l,
                "/muj-ucet",
                Some(Flash::Notice(t::auth::registered(l).into())),
            )
            .await
        }
        Err(err) => {
            tracing::warn!("account not created: {err}");
            let _ = session
                .insert(
                    REGISTRATION_FORM,
                    (email, vec![e::email_taken(l).to_owned()]),
                )
                .await;
            Redirect::to(&l.path("/registrace")).into_response()
        }
    }
}

/// The same answer whether or not the e-mail has an account.
pub async fn request_reset(
    State(state): State<AppState>,
    session: Session,
    ip: refractory::RemoteIp,
    uri: Uri,
    Form(fields): Form<Fields>,
) -> Response {
    let l = locale_of(&uri);
    if let Some(refused) = too_many_attempts(
        &state,
        &session,
        &ip,
        &crate::limits::PASSWORD_RESET,
        l,
        "/zapomenute-heslo",
    )
    .await
    {
        return refused;
    }
    let email = normalize_email(field(&fields, "email_address").unwrap_or_default());
    let user = sqlx::query!(
        r#"SELECT id AS "id!", email_address, password_hash FROM users WHERE email_address = ?"#,
        email
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();
    if let Some(user) = user
        && let Ok(hash) = user.password_hash.parse::<PasswordHash>()
    {
        let token = state.reset_tokens.issue(user.id, &hash, SystemTime::now());
        let url = format!(
            "{}{}?token={token}",
            crate::pages::base_url_for_mail(),
            l.path("/nove-heslo/odkaz")
        );
        use t::auth::reset as r;
        let text = format!(
            "{}\n\n{}\n\n{url}\n\n{}\n{}\n",
            r::mail_title(l),
            r::mail_intro(l),
            r::mail_expiry(l),
            r::mail_ignore(l)
        );
        tokio::spawn(crate::mail::send(crate::mail::Mail {
            to: user.email_address,
            subject: r::mail_subject(l).into(),
            text,
            html: None,
        }));
    }
    redirect_with(
        &session,
        l,
        "/prihlaseni",
        Some(Flash::Notice(t::auth::reset::sent(l).into())),
    )
    .await
}

/// The user a reset token is for, if it's still good.
pub async fn reset_user(state: &AppState, token: &str) -> Option<(i64, String)> {
    let claim = state.reset_tokens.decode(token, SystemTime::now()).ok()?;
    let id = claim.user_id();
    let user = sqlx::query!(
        "SELECT email_address, password_hash FROM users WHERE id = ?",
        id
    )
    .fetch_optional(&state.db)
    .await
    .ok()??;
    let hash: PasswordHash = user.password_hash.parse().ok()?;
    state.reset_tokens.confirm(&claim, &hash).ok()?;
    Some((id, user.email_address))
}

/// The e-mailed link: keeps the token in the session (out of the address bar).
pub async fn open_reset_link(
    session: Session,
    uri: Uri,
    Query(query): Query<Vec<(String, String)>>,
) -> Response {
    let l = locale_of(&uri);
    if let Some((_, token)) = query.iter().find(|(k, _)| k == "token") {
        let _ = session.insert(RESET_TOKEN, token.clone()).await;
    }
    Redirect::to(&l.path("/nove-heslo")).into_response()
}

pub async fn reset_token(session: &Session) -> Option<String> {
    session.get(RESET_TOKEN).await.ok().flatten()
}

pub async fn set_new_password(
    State(state): State<AppState>,
    session: Session,
    ip: refractory::RemoteIp,
    uri: Uri,
    Form(fields): Form<Fields>,
) -> Response {
    let l = locale_of(&uri);
    if let Some(refused) = too_many_attempts(
        &state,
        &session,
        &ip,
        &crate::limits::PASSWORD_UPDATE,
        l,
        "/nove-heslo",
    )
    .await
    {
        return refused;
    }
    let token = reset_token(&session).await.unwrap_or_default();
    let Some((user_id, _)) = reset_user(&state, &token).await else {
        let _ = session.remove::<String>(RESET_TOKEN).await;
        return redirect_with(
            &session,
            l,
            "/zapomenute-heslo",
            Some(Flash::Alert(t::auth::reset::invalid(l).into())),
        )
        .await;
    };
    let password = fields
        .iter()
        .find(|(k, _)| k == "password")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let confirmation = fields
        .iter()
        .find(|(k, _)| k == "password_confirmation")
        .map(|(_, v)| v.as_str());
    let problem = if password.is_empty() {
        Some(t::auth::reset::blank(l).to_owned())
    } else {
        let problems = password_problems(l, &password, confirmation);
        (!problems.is_empty()).then(|| problems.join(", "))
    };
    if let Some(problem) = problem {
        return redirect_with(&session, l, "/nove-heslo", Some(Flash::Alert(problem))).await;
    }
    let Ok(password) = Password::new(password) else {
        unreachable!("checked above")
    };
    let Ok(hash) = blocking(move || PasswordHash::new(&password)).await else {
        return redirect_with(
            &session,
            l,
            "/nove-heslo",
            Some(Flash::Alert(t::checkout::submit_error(l).into())),
        )
        .await;
    };
    let hash = hash.into_string();
    // A new password signs the account out everywhere.
    let saved = sqlx::query!(
        "UPDATE users SET password_hash = ?, session_epoch = session_epoch + 1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?",
        hash,
        user_id
    )
    .execute(&state.db)
    .await;
    if let Err(e) = saved {
        tracing::error!("password not saved: {e}");
        return redirect_with(
            &session,
            l,
            "/nove-heslo",
            Some(Flash::Alert(t::checkout::submit_error(l).into())),
        )
        .await;
    }
    let _ = session.remove::<String>(RESET_TOKEN).await;
    redirect_with(
        &session,
        l,
        "/prihlaseni",
        Some(Flash::Notice(t::auth::reset::done(l).into())),
    )
    .await
}

/// Only for a signed-in user; otherwise to the sign-in page (and back here after).
pub async fn require_user(session: &Session, v: &Visitor, here: &str) -> Result<i64, Response> {
    match &v.user {
        Some(user) => Ok(user.id),
        None => {
            let _ = session.insert(RETURN_TO, v.link(here)).await;
            Err(redirect_with(
                session,
                v.locale,
                "/prihlaseni",
                Some(Flash::Alert(t::auth::login_required(v.locale).into())),
            )
            .await)
        }
    }
}

const PHONE_PREFIXES: [&str; 2] = ["+420", "+43"];

pub async fn update_account(
    State(state): State<AppState>,
    session: Session,
    Extension(v): Extension<Visitor>,
    Form(fields): Form<Fields>,
) -> Response {
    let user_id = match require_user(&session, &v, "/muj-ucet/upravit").await {
        Ok(id) => id,
        Err(r) => return r,
    };
    let get = |name: &str| {
        fields
            .iter()
            .rev()
            .find(|(k, _)| *k == format!("user[{name}]"))
            .map(|(_, v)| v.trim().to_owned())
            .filter(|v| !v.is_empty())
    };
    let ticked = |name: &str| i64::from(get(name).as_deref() == Some("1"));
    let prefix = get("phone_prefix")
        .filter(|p| PHONE_PREFIXES.contains(&p.as_str()))
        .unwrap_or_else(|| PHONE_PREFIXES[0].to_owned());
    let phone = get("phone_number").map(|n| format!("{prefix} {n}"));
    let (first, last, street, city, zip, country) = (
        get("first_name"),
        get("last_name"),
        get("billing_street"),
        get("billing_city"),
        get("billing_zip"),
        get("billing_country"),
    );
    let (company, company_name, ico, dic) = (
        ticked("company_purchase"),
        get("company_name"),
        get("company_ico"),
        get("company_dic"),
    );
    let (different, d_street, d_city, d_zip, d_country) = (
        ticked("delivery_address_different"),
        get("delivery_street"),
        get("delivery_city"),
        get("delivery_zip"),
        get("delivery_country"),
    );
    let saved = sqlx::query!(
        "UPDATE users SET first_name = ?, last_name = ?, phone = ?, billing_street = ?, billing_city = ?, billing_zip = ?, billing_country = ?,
                          company_purchase = ?, company_name = ?, company_ico = ?, company_dic = ?,
                          delivery_address_different = ?, delivery_street = ?, delivery_city = ?, delivery_zip = ?, delivery_country = ?,
                          updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
        first, last, phone, street, city, zip, country, company, company_name, ico, dic, different, d_street, d_city, d_zip, d_country, user_id
    )
    .execute(&state.db)
    .await;
    let l = v.locale;
    match saved {
        Ok(_) => {
            redirect_with(
                &session,
                l,
                "/muj-ucet",
                Some(Flash::Notice(t::auth::profile_updated(l).into())),
            )
            .await
        }
        Err(e) => {
            tracing::error!("profile not saved: {e}");
            redirect_with(
                &session,
                l,
                "/muj-ucet/upravit",
                Some(Flash::Alert(t::checkout::submit_error(l).into())),
            )
            .await
        }
    }
}

/// Puts an earlier order's items back into the cart (those still sold).
pub async fn reorder(
    State(state): State<AppState>,
    session: Session,
    Extension(mut v): Extension<Visitor>,
    Path(id): Path<i64>,
) -> Response {
    let user_id = match require_user(&session, &v, "/muj-ucet").await {
        Ok(id) => id,
        Err(r) => return r,
    };
    let l = v.locale;
    let Some(items) = sqlx::query_scalar!(
        "SELECT items FROM orders WHERE id = ? AND user_id = ?",
        id,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten() else {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    };
    let lines: Vec<crate::orders::OrderLine> = serde_json::from_str(&items).unwrap_or_default();
    let (mut added, mut skipped) = (0, 0);
    for line in lines {
        match crate::cart_ops::add(
            &state,
            &session,
            &mut v,
            line.catalog_variant_id,
            line.quantity,
        )
        .await
        {
            Ok(()) => added += 1,
            Err(_) => skipped += 1,
        }
    }
    if added == 0 {
        redirect_with(
            &session,
            l,
            "/muj-ucet",
            Some(Flash::Alert(t::auth::reorder_failed(l).into())),
        )
        .await
    } else if skipped > 0 {
        redirect_with(
            &session,
            l,
            "/kosik",
            Some(Flash::Notice(t::auth::reorder_partial(l, skipped))),
        )
        .await
    } else {
        redirect_with(
            &session,
            l,
            "/kosik",
            Some(Flash::Notice(t::auth::reorder_success(l).into())),
        )
        .await
    }
}

fn avatar_dir(state: &AppState) -> std::path::PathBuf {
    state.storage.join("avatars")
}

/// The signed-in user's own profile picture (never anyone else's).
pub async fn show_avatar(
    State(state): State<AppState>,
    Extension(v): Extension<Visitor>,
) -> Response {
    let Some(user) = &v.user else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(file) = sqlx::query_scalar!("SELECT avatar_path FROM users WHERE id = ?", user.id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .flatten()
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    // avatar_path is "/muj-ucet/profilovy-obrazek?v=<file name>".
    let Some(name) = file
        .split("v=")
        .nth(1)
        .filter(|n| n.chars().all(|c| c.is_ascii_alphanumeric() || c == '.'))
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(bytes) = tokio::fs::read(avatar_dir(&state).join(name)).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mime = spliceosome::Format::detect(&bytes).map_or("application/octet-stream", |f| f.mime());
    (
        [
            (header::CONTENT_TYPE, mime),
            (header::CACHE_CONTROL, "private, max-age=31536000"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (
                header::CONTENT_SECURITY_POLICY,
                "default-src 'none'; sandbox",
            ),
        ],
        bytes,
    )
        .into_response()
}

pub async fn upload_avatar(
    State(state): State<AppState>,
    session: Session,
    ip: refractory::RemoteIp,
    Extension(v): Extension<Visitor>,
    mut form: Multipart,
) -> Response {
    use t::auth::avatar_errors as e;
    let user_id = match require_user(&session, &v, "/muj-ucet/upravit").await {
        Ok(id) => id,
        Err(r) => return r,
    };
    let l = v.locale;
    let back = |message: String, error: bool| {
        let session = session.clone();
        async move {
            redirect_with(
                &session,
                l,
                "/muj-ucet/upravit",
                Some(if error {
                    Flash::Alert(message)
                } else {
                    Flash::Notice(message)
                }),
            )
            .await
        }
    };
    if state
        .limiter
        .check(&crate::limits::AVATAR, ip.key())
        .await
        .is_err()
    {
        return back(e::rate_limited(l).into(), true).await;
    }
    let mut upload = None;
    while let Ok(Some(part)) = form.next_field().await {
        if part.name() == Some("avatar") {
            upload = part.bytes().await.ok();
        }
    }
    let Some(bytes) = upload.filter(|b| !b.is_empty()) else {
        return back(e::none_selected(l).into(), true).await;
    };
    let clean = match spliceosome::sanitize(&bytes, &spliceosome::Limits::DEFAULT) {
        Ok(clean) => clean,
        Err(err) => {
            let message = match err {
                spliceosome::Error::TooLarge { .. } => e::too_large(l),
                spliceosome::Error::TooManyPixels { .. } => e::too_many_pixels(l),
                spliceosome::Error::UnsupportedFormat | spliceosome::Error::Animated { .. } => {
                    e::unsupported_type(l)
                }
                _ => e::unreadable(l),
            };
            return back(message.into(), true).await;
        }
    };
    // Named after its content, so a new picture gets a new address (and can be cached for a year).
    use sha2::Digest;
    let digest = sha2::Sha256::digest(clean.bytes());
    let hex: String = digest.iter().take(8).map(|b| format!("{b:02x}")).collect();
    let name = format!("{user_id}x{hex}.{}", clean.extension());
    let dir = avatar_dir(&state);
    if let Err(err) = async {
        tokio::fs::create_dir_all(&dir).await?;
        tokio::fs::write(dir.join(&name), clean.bytes()).await
    }
    .await
    {
        tracing::error!("avatar not saved: {err}");
        return back(t::checkout::submit_error(l).into(), true).await;
    }
    let old = sqlx::query_scalar!("SELECT avatar_path FROM users WHERE id = ?", user_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .flatten();
    let path = format!("/muj-ucet/profilovy-obrazek?v={name}");
    let _ = sqlx::query!(
        "UPDATE users SET avatar_path = ? WHERE id = ?",
        path,
        user_id
    )
    .execute(&state.db)
    .await;
    remove_avatar_file(&state, old.as_deref()).await;
    back(t::auth::avatar_updated(l).into(), false).await
}

async fn remove_avatar_file(state: &AppState, path: Option<&str>) {
    if let Some(name) = path
        .and_then(|p| p.split("v=").nth(1))
        .filter(|n| n.chars().all(|c| c.is_ascii_alphanumeric() || c == '.'))
    {
        let _ = tokio::fs::remove_file(avatar_dir(state).join(name)).await;
    }
}

pub async fn remove_avatar(
    State(state): State<AppState>,
    session: Session,
    Extension(v): Extension<Visitor>,
) -> Response {
    let user_id = match require_user(&session, &v, "/muj-ucet/upravit").await {
        Ok(id) => id,
        Err(r) => return r,
    };
    let old = sqlx::query_scalar!("SELECT avatar_path FROM users WHERE id = ?", user_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .flatten();
    let _ = sqlx::query!("UPDATE users SET avatar_path = NULL WHERE id = ?", user_id)
        .execute(&state.db)
        .await;
    remove_avatar_file(&state, old.as_deref()).await;
    redirect_with(
        &session,
        v.locale,
        "/muj-ucet/upravit",
        Some(Flash::Notice(t::auth::avatar_removed(v.locale).into())),
    )
    .await
}
