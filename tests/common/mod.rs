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
    /// The HTML without Leptos's `<!>` markers (empty comments between
    /// pieces of text, which browsers ignore).
    pub fn html(&self) -> String {
        self.body.replace("<!>", "")
    }

    pub fn assert_contains(&self, needle: &str) {
        let html = self.html();
        assert!(html.contains(needle), "expected {needle:?} in:\n{html}");
    }

    pub fn assert_lacks(&self, needle: &str) {
        let html = self.html();
        assert!(!html.contains(needle), "did not expect {needle:?} in:\n{html}");
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

    /// With the fixture data plus two accounts (`accounts.sql`, password
    /// "heslo12345") and a storage folder of its own, so uploaded avatars
    /// don't end up among the fixtures.
    pub async fn with_accounts() -> Self {
        let db = db::in_memory().await.expect("database");
        for sql in [
            include_str!("../fixtures/data.sql"),
            include_str!("../fixtures/accounts.sql"),
        ] {
            sqlx::raw_sql(sql).execute(&db).await.expect("fixtures");
        }
        let storage = std::env::temp_dir().join(format!("pila-test-{}", random_suffix()));
        copy_dir(&fixture_storage(), &storage);
        Self::build(db, default_services(), storage)
    }

    /// With an empty database.
    pub async fn empty() -> Self {
        Self::with_db(db::in_memory().await.expect("database"))
    }

    fn with_db(db: db::Db) -> Self {
        Self::with_services(db, default_services())
    }

    fn with_services(db: db::Db, services: Services) -> Self {
        Self::build(db, services, fixture_storage())
    }

    fn build(db: db::Db, services: Services, storage_root: PathBuf) -> Self {
        let mut config = Config::from_source_tree(false);
        config.storage_root = storage_root;
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
/// token the last page's forms carried, like a real form would.
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
        // Every form carries the session's token in a hidden field.
        const FIELD: &str = r#"name="csrf_token" value=""#;
        if let Some(start) = page.body.find(FIELD) {
            let rest = &page.body[start + FIELD.len()..];
            self.token = rest[..rest.find('"').unwrap()].to_string();
        }
        page
    }

    /// Submits a form with the page's token.
    pub async fn post(&mut self, path: &str, fields: &[(&str, &str)]) -> Page {
        let mut all = vec![("csrf_token", self.token.clone())];
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

    /// Submits a multipart form with one file, plus the page's token.
    pub async fn post_file(&mut self, path: &str, fields: &[(&str, &str)], file: (&str, &str, &[u8])) -> Page {
        let boundary = "----pilaTestBoundary";
        let mut body: Vec<u8> = Vec::new();
        let mut all = vec![("csrf_token", self.token.clone())];
        all.extend(fields.iter().map(|(k, v)| (*k, v.to_string())));
        for (name, value) in all {
            body.extend(
                format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n")
                    .as_bytes(),
            );
        }
        let (field, file_name, bytes) = file;
        body.extend(
            format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{field}\"; filename=\"{file_name}\"\r\nContent-Type: application/octet-stream\r\n\r\n")
                .as_bytes(),
        );
        body.extend_from_slice(bytes);
        body.extend(format!("\r\n--{boundary}--\r\n").as_bytes());
        let request = Request::post(path).header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        );
        self.send(request, Body::from(body)).await
    }

    /// Signs in with the fixture password.
    pub async fn sign_in(&mut self, email: &str) -> Page {
        self.get("/prihlaseni").await;
        self.post("/prihlaseni", &[("email_address", email), ("password", "heslo12345")])
            .await
    }

    /// The Cookie header the browser sends now ("a=1; b=2").
    pub fn cookie_header(&self) -> &str {
        self.cookie.as_deref().unwrap_or_default()
    }

    /// Sends these cookies from now on, as if copied from another browser.
    pub fn set_cookie_header(&mut self, cookies: &str) {
        self.cookie = Some(cookies.to_string());
    }

    /// Forgets one cookie, as when a browser session ends.
    pub fn drop_cookie(&mut self, name: &str) {
        self.keep_cookie(&format!("{name}=; max-age=0"));
    }

    /// Keeps a Set-Cookie like a browser: replaces the cookie of that name,
    /// or forgets it when the value is empty or already expired.
    fn keep_cookie(&mut self, set_cookie: &str) {
        let Some((name, value)) = set_cookie.split(';').next().and_then(|pair| pair.split_once('=')) else {
            return;
        };
        let expired = value.is_empty() || set_cookie.to_ascii_lowercase().contains("max-age=0");
        let mut jar: Vec<String> = self
            .cookie
            .as_deref()
            .unwrap_or_default()
            .split("; ")
            .filter(|pair| !pair.is_empty() && pair.split('=').next() != Some(name))
            .map(str::to_string)
            .collect();
        if !expired {
            jar.push(format!("{name}={value}"));
        }
        self.cookie = (!jar.is_empty()).then(|| jar.join("; "));
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
        for set in response.headers().get_all(header::SET_COOKIE) {
            self.keep_cookie(set.to_str().unwrap_or_default());
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

fn default_services() -> Services {
    Services {
        rates: ExchangeRates::fixed(25.0),
        routes: RouteService::new(None),
    }
}

fn fixture_storage() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/storage")
}

fn random_suffix() -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    format!("{}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn copy_dir(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).expect("storage folder");
    for entry in std::fs::read_dir(from).expect("fixture storage") {
        let entry = entry.expect("entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy");
        }
    }
}
