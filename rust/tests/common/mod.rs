//! Shared by the test files: the app with a fresh in-memory database filled
//! from `fixtures/data.sql`, and a helper to send it requests.

#![allow(dead_code)]

use std::path::PathBuf;

use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode, header};
use http_body_util::BodyExt;
use piladerflice::money::ExchangeRates;
use piladerflice::{AppState, Config, app, db};
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

    /// With an empty database.
    pub async fn empty() -> Self {
        Self::with_db(db::in_memory().await.expect("database"))
    }

    fn with_db(db: db::Db) -> Self {
        let mut config = Config::from_source_tree(false);
        config.storage_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/storage");
        let state = AppState::with(config, db, ExchangeRates::fixed(25.0)).expect("app state");
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

/// One request against a fresh app with the fixture data.
pub async fn get(path: &str) -> Page {
    TestApp::new().await.get(path).await
}
