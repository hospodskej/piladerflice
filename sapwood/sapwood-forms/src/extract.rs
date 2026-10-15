//! Reading forms from requests, after checking they aren't forged.

use axum::extract::{FromRequest, Multipart, Request};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use sapwood_session::csrf::{self, RequestToken};
use serde::de::DeserializeOwned;

use crate::{CSRF_FIELD, FormData, MAX_FORM_BYTES, UploadForm, UploadedFile};

/// A refused form: forged, sent from another site, too big or unreadable.
/// The response is a short text with the status; it carries this value in
/// its extensions, so a middleware can swap in a page of your own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rejected {
    pub status: StatusCode,
    pub reason: &'static str,
}

impl Rejected {
    const fn new(status: StatusCode, reason: &'static str) -> Self {
        Self { status, reason }
    }
}

impl IntoResponse for Rejected {
    fn into_response(self) -> Response {
        let mut response = (self.status, self.reason).into_response();
        response.extensions_mut().insert(self);
        response
    }
}

/// The form's fields as `T` (`#[derive(Deserialize)]`). Use `String` for
/// text and number fields and check them yourself, so a wrong value
/// becomes a message on the form rather than a refused request.
#[derive(Debug, Clone)]
pub struct Form<T>(pub T);

impl<S: Send + Sync, T: DeserializeOwned> FromRequest<S> for Form<T> {
    type Rejection = Rejected;

    async fn from_request(request: Request, state: &S) -> Result<Self, Rejected> {
        let form = FormData::from_request(request, state).await?;
        form.deserialize().map(Form).map_err(|error| {
            tracing::warn!("unreadable form: {error}");
            Rejected::new(StatusCode::UNPROCESSABLE_ENTITY, "unreadable form")
        })
    }
}

impl<S: Send + Sync> FromRequest<S> for FormData {
    type Rejection = Rejected;

    async fn from_request(request: Request, _state: &S) -> Result<Self, Rejected> {
        let (parts, body) = request.into_parts();
        let bytes = axum::body::to_bytes(body, MAX_FORM_BYTES)
            .await
            .map_err(|_| Rejected::new(StatusCode::PAYLOAD_TOO_LARGE, "form too large"))?;
        let form = FormData::parse(&bytes);
        verify(&parts.headers, &parts.extensions, form.get(CSRF_FIELD))?;
        Ok(form)
    }
}

impl<S: Send + Sync> FromRequest<S> for UploadForm {
    type Rejection = Rejected;

    async fn from_request(request: Request, state: &S) -> Result<Self, Rejected> {
        let multipart = request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("multipart/form-data"));
        if !multipart {
            let fields = FormData::from_request(request, state).await?;
            return Ok(UploadForm {
                fields,
                files: Vec::new(),
            });
        }
        let headers = request.headers().clone();
        let token = request.extensions().get::<RequestToken>().cloned();
        let mut multipart = Multipart::from_request(request, state)
            .await
            .map_err(|_| Rejected::new(StatusCode::BAD_REQUEST, "unreadable form"))?;

        let too_large = || Rejected::new(StatusCode::PAYLOAD_TOO_LARGE, "form too large");
        let mut pairs = Vec::new();
        let mut files = Vec::new();
        while let Some(field) = multipart.next_field().await.map_err(|_| too_large())? {
            let name = field.name().unwrap_or_default().to_string();
            match field.file_name().map(str::to_string) {
                Some(file_name) => {
                    let bytes = field.bytes().await.map_err(|_| too_large())?;
                    files.push(UploadedFile {
                        field: name,
                        file_name,
                        bytes: bytes.to_vec(),
                    });
                }
                None => pairs.push((name, field.text().await.map_err(|_| too_large())?)),
            }
        }
        let fields = FormData::from_pairs(pairs);
        let mut extensions = axum::http::Extensions::new();
        if let Some(token) = token {
            extensions.insert(token);
        }
        verify(&headers, &extensions, fields.get(CSRF_FIELD))?;
        Ok(UploadForm { fields, files })
    }
}

/// The Origin must be the site itself and the token the session's.
fn verify(headers: &HeaderMap, extensions: &axum::http::Extensions, form_token: Option<&str>) -> Result<(), Rejected> {
    let forged = Rejected::new(StatusCode::UNPROCESSABLE_ENTITY, "forged form");
    let origin = headers.get(header::ORIGIN).and_then(|value| value.to_str().ok());
    if !csrf::origin_allowed(origin, &base_url(headers)) {
        tracing::warn!("form from another site refused (Origin {origin:?})");
        return Err(forged);
    }
    let Some(token) = extensions.get::<RequestToken>() else {
        tracing::error!("forms need sapwood_session::SessionLayer on the router");
        return Err(forged);
    };
    let header_token = headers.get(csrf::HEADER).and_then(|value| value.to_str().ok());
    if !token.verify(form_token.or(header_token).unwrap_or_default()) {
        tracing::warn!("form without a valid CSRF token refused");
        return Err(forged);
    }
    Ok(())
}

/// "https://example.com": the scheme (from `X-Forwarded-Proto` behind a
/// proxy) and host the visitor used.
fn base_url(headers: &HeaderMap) -> String {
    let header = |name: &str| headers.get(name).and_then(|value| value.to_str().ok());
    let scheme = header("x-forwarded-proto")
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .unwrap_or("http");
    format!("{scheme}://{}", header("host").unwrap_or("localhost"))
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::body::Body;
    use axum::routing::{get, post};
    use http_body_util::BodyExt;
    use sapwood_session::{Session, SessionLayer};
    use tower::ServiceExt;

    use super::*;

    fn app() -> Router {
        Router::new()
            .route(
                "/token",
                get(|session: Session<()>| async move { session.csrf_token() }),
            )
            .route(
                "/plain",
                post(|form: FormData| async move { form.get("name").unwrap_or_default().to_string() }),
            )
            .route(
                "/typed",
                post(|Form(form): Form<std::collections::HashMap<String, String>>| async move { form["name"].clone() }),
            )
            .route(
                "/upload",
                post(|form: UploadForm| async move {
                    format!(
                        "{} {:?}",
                        form.get("name").unwrap_or_default(),
                        form.file("photo").map(|f| f.bytes.len())
                    )
                }),
            )
            .layer(SessionLayer::<()>::new(&[3; 64]).secure(false))
    }

    /// The session cookie and its token.
    async fn session() -> (String, String) {
        let response = app()
            .oneshot(Request::get("/token").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let cookie = response.headers()[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_string();
        let token = String::from_utf8(response.into_body().collect().await.unwrap().to_bytes().to_vec()).unwrap();
        (cookie, token)
    }

    async fn send(
        path: &str,
        cookie: &str,
        content_type: &str,
        body: String,
        origin: Option<&str>,
    ) -> (StatusCode, String) {
        let mut request = Request::post(path)
            .header(header::COOKIE, cookie)
            .header(header::CONTENT_TYPE, content_type)
            .header(header::HOST, "shop.example");
        if let Some(origin) = origin {
            request = request.header(header::ORIGIN, origin);
        }
        let response = app().oneshot(request.body(Body::from(body)).unwrap()).await.unwrap();
        let status = response.status();
        (
            status,
            String::from_utf8(response.into_body().collect().await.unwrap().to_bytes().to_vec()).unwrap(),
        )
    }

    const URLENCODED: &str = "application/x-www-form-urlencoded";

    #[tokio::test]
    async fn forms_with_the_token_get_through() {
        let (cookie, token) = session().await;
        let body = format!("name=Jana&csrf_token={token}");
        assert_eq!(
            send("/plain", &cookie, URLENCODED, body.clone(), None).await,
            (StatusCode::OK, "Jana".into())
        );
        assert_eq!(
            send("/typed", &cookie, URLENCODED, body.clone(), Some("http://shop.example"))
                .await
                .1,
            "Jana"
        );
    }

    #[tokio::test]
    async fn forged_forms_are_refused() {
        let (cookie, token) = session().await;
        let refused = (StatusCode::UNPROCESSABLE_ENTITY, "forged form".to_string());
        assert_eq!(
            send("/plain", &cookie, URLENCODED, "name=Jana".into(), None).await,
            refused
        );
        assert_eq!(
            send("/plain", "", URLENCODED, format!("name=Jana&csrf_token={token}"), None).await,
            refused
        );
        assert_eq!(
            send(
                "/plain",
                &cookie,
                URLENCODED,
                format!("name=Jana&csrf_token={token}"),
                Some("https://evil.example")
            )
            .await,
            refused
        );
    }

    #[tokio::test]
    async fn uploads_bring_fields_and_files() {
        let (cookie, token) = session().await;
        let body = format!(
            "--XX\r\nContent-Disposition: form-data; name=\"csrf_token\"\r\n\r\n{token}\r\n\
             --XX\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\nJana\r\n\
             --XX\r\nContent-Disposition: form-data; name=\"photo\"; filename=\"a.png\"\r\nContent-Type: image/png\r\n\r\nPNGDATA\r\n\
             --XX--\r\n"
        );
        let multipart = "multipart/form-data; boundary=XX";
        assert_eq!(
            send("/upload", &cookie, multipart, body.clone(), None).await.1,
            "Jana Some(7)"
        );
        let forged = body.replace(&token, "nope");
        assert_eq!(
            send("/upload", &cookie, multipart, forged, None).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        // A plain form reaches the same handler.
        let plain = format!("name=Eva&csrf_token={token}");
        assert_eq!(send("/upload", &cookie, URLENCODED, plain, None).await.1, "Eva None");
    }
}
