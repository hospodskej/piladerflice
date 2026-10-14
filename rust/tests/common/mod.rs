//! Shared by the test files: the app with a fresh in-memory database filled
//! from `fixtures/data.sql`, and a helper to send it requests.

#![allow(dead_code)]

use std::path::PathBuf;

use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode, header};
use http_body_util::BodyExt;
use piladerflice::money::ExchangeRates;
use piladerflice::shop::delivery::RouteService;
use piladerflice::{AppState, Config, Services, app, db};
use tower::ServiceExt;

pub struct Page {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: String,
}

impl Page {
    pub fn assert_contains(&self, needle: &str) {
        assert!(self.body.contains(needle), "expected {needle:?} in:\n{}", self.body);
    }

    pub fn assert_lacks(&self, needle: &str) {
        assert!(
            !self.body.contains(needle),
            "did not expect {needle:?} in:\n{}",
            self.body
        );
    }
}

pub struct TestApp {
    pub state: AppState,
}

impl TestApp {
    /// With the fixture data, 25 CZK per EUR.
    pub async fn new() -> Self {
        let db = db::in_memory().await.expect("database");
        sqlx::raw_sql(include_str!("../fixtures/data.sql"))
            .execute(&db)
            .await
            .expect("fixtures");
        Self::with_db(db)
    }

    /// With the fixture data and a fake delivery distance service: every
    /// address is `km` kilometres away.
    pub async fn with_distance(km: f64) -> Self {
        let db = db::in_memory().await.expect("database");
        sqlx::raw_sql(include_str!("../fixtures/data.sql"))
            .execute(&db)
            .await
            .expect("fixtures");
        let fetch: piladerflice::shop::delivery::route_service::Fetcher = std::sync::Arc::new(move |url: String| {
            Box::pin(async move {
                Ok(if url.contains("/geocode/") {
                    serde_json::json!({ "features": [{ "geometry": { "coordinates": [16.6, 49.2] } }] })
                } else {
                    serde_json::json!({ "features": [{ "properties": { "summary": { "distance": km * 1000.0 } } }] })
                })
            })
        });
        let routes = RouteService::with_fetcher(Some("test-key".into()), fetch);
        Self::with_services(
            db,
            Services {
                rates: ExchangeRates::fixed(25.0),
                routes,
            },
        )
    }

    /// With an empty database.
    pub async fn empty() -> Self {
        Self::with_db(db::in_memory().await.expect("database"))
    }

    fn with_db(db: db::Db) -> Self {
        Self::with_services(
            db,
            Services {
                rates: ExchangeRates::fixed(25.0),
                routes: RouteService::new(None),
            },
        )
    }

    fn with_services(db: db::Db, services: Services) -> Self {
        let mut config = Config::from_source_tree(false);
        config.storage_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/storage");
        let state = AppState::with(config, db, services).expect("app state");
        Self { state }
    }

    pub async fn get(&self, path: &str) -> Page {
        let request = Request::get(path)
            .header(header::HOST, "piladerflice.test")
            .body(Body::empty())
            .unwrap();
        let response = app(self.state.clone()).oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        Page {
            status,
            headers,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        }
    }
}

/// A browser: keeps the session cookie between requests and sends the form
/// token the last page carried, like a real form would.
pub struct Browser<'a> {
    app: &'a TestApp,
    cookie: Option<String>,
    token: String,
    pub turbo: bool,
    /// Sent as the Origin header of form posts.
    pub origin: Option<String>,
}

impl TestApp {
    pub fn browser(&self) -> Browser<'_> {
        Browser {
            app: self,
            cookie: None,
            token: String::new(),
            turbo: false,
            origin: None,
        }
    }

    pub fn outbox(&self) -> Vec<piladerflice::mail::Email> {
        self.state.mailer.outbox()
    }
}

impl Browser<'_> {
    pub async fn get(&mut self, path: &str) -> Page {
        let page = self.send(Request::get(path), Body::empty()).await;
        if let Some(start) = page.body.find(r#"<meta name="csrf-token" content=""#) {
            let rest = &page.body[start + 33..];
            self.token = rest[..rest.find('"').unwrap()].to_string();
        }
        page
    }

    /// Submits a form with the page's token.
    pub async fn post(&mut self, path: &str, fields: &[(&str, &str)]) -> Page {
        let mut all = vec![("authenticity_token", self.token.clone())];
        all.extend(fields.iter().map(|(k, v)| (*k, v.to_string())));
        self.post_raw(path, &all).await
    }

    /// Submits exactly these fields (no token added).
    pub async fn post_raw(&mut self, path: &str, fields: &[(&str, String)]) -> Page {
        let body = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(fields)
            .finish();
        let mut request = Request::post(path).header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
        if self.turbo {
            request = request.header(header::ACCEPT, "text/vnd.turbo-stream.html, text/html");
        }
        if let Some(origin) = &self.origin {
            request = request.header(header::ORIGIN, origin.clone());
        }
        self.send(request, Body::from(body)).await
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    async fn send(&mut self, mut request: axum::http::request::Builder, body: Body) -> Page {
        request = request.header(header::HOST, "piladerflice.test");
        if let Some(cookie) = &self.cookie {
            request = request.header(header::COOKIE, cookie.clone());
        }
        let response = app(self.app.state.clone())
            .oneshot(request.body(body).unwrap())
            .await
            .unwrap();
        if let Some(set) = response.headers().get(header::SET_COOKIE) {
            self.cookie = set
                .to_str()
                .ok()
                .and_then(|value| value.split(';').next())
                .map(str::to_string);
        }
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        Page {
            status,
            headers,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        }
    }
}

impl Page {
    pub fn location(&self) -> &str {
        self.headers
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
    }
}

/// One request against a fresh app with the fixture data.
pub async fn get(path: &str) -> Page {
    TestApp::new().await.get(path).await
}
