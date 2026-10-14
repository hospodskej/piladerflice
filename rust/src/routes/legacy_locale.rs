//! Links shared before the language moved into the URL looked like
//! `/kontakt?locale=de`. They are sent to the new address for good (301).

use axum::extract::Request;
use axum::http::{Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::locale::{self, Locale};
use crate::request::OriginalUri;

pub async fn redirect(request: Request, next: Next) -> Response {
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
