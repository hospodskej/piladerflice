//! What every page needs to know about the request it answers: the
//! language, the path with and without the language prefix, and the site's
//! own address (for absolute links in share cards and the sitemap).
//!
//! The language prefix is taken off the URL before routing (see
//! [`strip_locale_prefix`]), so each route is written once and answers both
//! `/kontakt` and `/at/kontakt`.

use axum::extract::{FromRequestParts, Request};
use axum::http::request::Parts;
use axum::http::{HeaderMap, Uri, header};

use crate::AppState;
use crate::accounts::current_user::CurrentUser;
use crate::locale::{self, Locale};
use crate::routes::AppError;
use crate::session::Session;

/// The path as the visitor asked for it, before the language prefix was removed.
#[derive(Clone, Debug)]
pub struct OriginalUri(pub Uri);

/// Paths that exist in one language only (Czech admin, files, machine-read
/// URLs). Under /at they are left alone, so they don't match and get a 404.
const UNLOCALIZED: [&str; 5] = ["/admin", "/assets", "/robots.txt", "/sitemap.xml", "/up"];

/// Removes the /at prefix and remembers the language, before the router
/// looks at the path. A trailing slash is ignored too ("/eshop/" is
/// "/eshop"), as in Rails.
pub fn strip_locale_prefix(mut request: Request) -> Request {
    let original = request.uri().clone();
    let path = original.path();
    let locale = locale::prefix_locale(path);

    let mut routed = path.to_string();
    if locale.is_some() {
        let stripped = locale::strip(path);
        let unlocalized = UNLOCALIZED
            .iter()
            .any(|p| stripped == *p || stripped.starts_with(&format!("{p}/")));
        if !unlocalized {
            routed = stripped;
        }
    }
    if routed.len() > 1 && routed.ends_with('/') {
        routed = routed.trim_end_matches('/').to_string();
        if routed.is_empty() {
            routed = "/".into();
        }
    }
    if routed != path {
        let path_and_query = match original.query() {
            Some(query) => format!("{routed}?{query}"),
            None => routed,
        };
        if let Ok(uri) = path_and_query.parse() {
            *request.uri_mut() = uri;
        }
    }

    request.extensions_mut().insert(locale.unwrap_or(Locale::Cs));
    request.extensions_mut().insert(OriginalUri(original));
    request
}

#[derive(Clone, Debug)]
pub struct Req {
    pub locale: Locale,
    /// As requested: "/at/kontakt"
    pub path: String,
    /// Without the language prefix: "/kontakt"
    pub plain_path: String,
    pub query: Option<String>,
    /// "https://piladerflice.cz"
    pub base_url: String,
    pub headers: HeaderMap,
    /// This visitor's session (cart, checkout, form token...).
    pub session: Session,
    /// The signed-in customer, if any.
    pub user: Option<CurrentUser>,
}

impl Req {
    pub fn query_pairs(&self) -> Vec<(String, String)> {
        self.query
            .as_deref()
            .map(|query| form_urlencoded::parse(query.as_bytes()).into_owned().collect())
            .unwrap_or_default()
    }

    pub fn query_param(&self, name: &str) -> Option<String> {
        self.query_pairs()
            .into_iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    }

    /// A site path in the current language.
    pub fn path_for(&self, path: &str) -> String {
        locale::localized_path(path, self.locale)
    }

    /// The visitor's address as the reverse proxy reports it (for the login
    /// record; informational only).
    pub fn client_ip(&self) -> Option<String> {
        let header = |name: &str| self.headers.get(name).and_then(|value| value.to_str().ok());
        header("x-forwarded-for")
            .and_then(|list| list.split(',').next())
            .or_else(|| header("x-real-ip"))
            .map(|ip| ip.trim().to_string())
            .filter(|ip| !ip.is_empty())
    }

    pub fn user_agent(&self) -> Option<String> {
        self.headers
            .get(axum::http::header::USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string)
    }

    /// Whether the browser asked for a Turbo Stream (a form submitted by
    /// Turbo), rather than a whole page.
    pub fn wants_turbo_stream(&self) -> bool {
        self.headers
            .get(axum::http::header::ACCEPT)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|accept| accept.contains("text/vnd.turbo-stream.html"))
    }
}

/// The signed-in user, once loaded for a request (extractors may run twice).
#[derive(Clone)]
struct LoadedUser(Option<CurrentUser>);

impl FromRequestParts<AppState> for Req {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let original = parts
            .extensions
            .get::<OriginalUri>()
            .map(|uri| uri.0.clone())
            .unwrap_or_else(|| parts.uri.clone());
        let locale = parts.extensions.get::<Locale>().copied().unwrap_or(Locale::Cs);
        // Like Rails, "/eshop/" is "/eshop" (canonical links, breadcrumbs).
        let raw = original.path();
        let path = if raw.len() > 1 { raw.trim_end_matches('/') } else { raw };
        let path = if path.is_empty() {
            "/".to_string()
        } else {
            path.to_string()
        };
        let session = parts
            .extensions
            .get::<Session>()
            .cloned()
            .unwrap_or_else(|| Session::new(Default::default()));
        let user = match parts.extensions.get::<LoadedUser>() {
            Some(loaded) => loaded.0.clone(),
            None => {
                let user = CurrentUser::load(&state.db, session.login()).await?;
                parts.extensions.insert(LoadedUser(user.clone()));
                user
            }
        };
        Ok(Req {
            locale,
            plain_path: locale::strip(&path),
            path,
            query: original.query().map(str::to_string),
            base_url: base_url(&parts.headers),
            headers: parts.headers.clone(),
            session,
            user,
        })
    }
}

/// Scheme and host the visitor used. Behind a reverse proxy (nginx, Caddy)
/// the scheme comes from X-Forwarded-Proto.
fn base_url(headers: &HeaderMap) -> String {
    let header = |name| headers.get(name).and_then(|value| value.to_str().ok());
    let scheme = header("x-forwarded-proto")
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .unwrap_or("http");
    let host = header(header::HOST.as_str()).unwrap_or("localhost");
    format!("{scheme}://{host}")
}
