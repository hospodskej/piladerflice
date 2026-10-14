//! All URLs of the site. Public pages answer in both languages: `/kontakt`
//! (Czech) and `/at/kontakt` (German) reach the same handler, which reads the
//! language from [`Req`](crate::request::Req).

mod errors;
mod pages;
mod system;

use axum::Router;
use axum::extract::Request;
use axum::http::{Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use tower_http::trace::TraceLayer;

use crate::AppState;
use crate::assets;
use crate::locale::{self, Locale};
use crate::request::OriginalUri;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/up", get(system::health))
        .route("/robots.txt", get(system::robots))
        .route("/assets/{*path}", get(assets::serve))
        .route("/obchodni-podminky", get(pages::terms))
        .route("/zasady-cookies", get(pages::cookies_policy))
        .route("/zasady-ochrany-osobnich-udaju", get(pages::privacy_policy))
        .route("/sortiment/palivove-drevo", pages::sortiment("palivove_drevo"))
        .route("/sortiment/stavebni-rezivo", pages::sortiment("stavebni_rezivo"))
        .route("/sortiment/truhlarske-rezivo", pages::sortiment("truhlarske_rezivo"))
        .route("/sortiment/okrasne-kamenivo", pages::sortiment("okrasne_kamenivo"))
        .route("/sortiment/vyrobni-zbytky", pages::sortiment("vyrobni_zbytky"))
        .fallback(errors::not_found)
        .layer(middleware::from_fn(redirect_legacy_locale_param))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Links shared before the language moved into the URL looked like
/// `/kontakt?locale=de`. Send them to the new address for good (301).
async fn redirect_legacy_locale_param(request: Request, next: Next) -> Response {
    if request.method() == Method::GET || request.method() == Method::HEAD {
        let original = request
            .extensions()
            .get::<OriginalUri>()
            .map(|uri| uri.0.clone())
            .unwrap_or_else(|| request.uri().clone());
        let pairs: Vec<(String, String)> = original
            .query()
            .map(|query| form_urlencoded::parse(query.as_bytes()).into_owned().collect())
            .unwrap_or_default();
        let legacy = pairs
            .iter()
            .find(|(key, _)| key == "locale")
            .and_then(|(_, value)| Locale::from_code(value));
        if let (Some(target_locale), None) = (legacy, locale::prefix_locale(original.path())) {
            let mut target = locale::localized_path(original.path(), target_locale);
            let rest: Vec<_> = pairs.iter().filter(|(key, _)| key != "locale").collect();
            if !rest.is_empty() {
                target.push('?');
                target.push_str(
                    &form_urlencoded::Serializer::new(String::new())
                        .extend_pairs(rest)
                        .finish(),
                );
            }
            return (StatusCode::MOVED_PERMANENTLY, [(header::LOCATION, target)]).into_response();
        }
    }
    next.run(request).await
}
