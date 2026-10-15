//! Answering `/assets/...` requests with Axum.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;

use crate::Assets;

impl Assets {
    /// A router serving the files under the URL prefix; merge it into your
    /// app: `app.merge(Assets::router(assets.clone()))`.
    pub fn router<S: Clone + Send + Sync + 'static>(assets: Arc<Assets>) -> Router<S> {
        let route = format!("{}/{{*path}}", assets.url_prefix());
        Router::new().route(&route, get(serve)).with_state(assets)
    }

    /// The response for a path under the URL prefix: the file, cached for
    /// a year when asked for by its fingerprinted name, otherwise `no-cache`;
    /// 404 for an unknown path.
    pub async fn response(&self, requested: &str) -> Response {
        let Some(found) = self.find(requested) else {
            return StatusCode::NOT_FOUND.into_response();
        };
        let Ok(bytes) = tokio::fs::read(&found.file).await else {
            return StatusCode::NOT_FOUND.into_response();
        };
        let mime = match found.file.extension().and_then(|ext| ext.to_str()) {
            Some("js" | "mjs") => "text/javascript".to_string(),
            _ => mime_guess::from_path(&found.file).first_or_octet_stream().to_string(),
        };
        let cache = if found.fingerprinted {
            "public, max-age=31536000, immutable"
        } else {
            "no-cache"
        };
        let mut response = Response::new(Body::from(bytes));
        let headers = response.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&mime).unwrap_or(HeaderValue::from_static("application/octet-stream")),
        );
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(cache));
        response
    }
}

async fn serve(State(assets): State<Arc<Assets>>, Path(requested): Path<String>) -> Response {
    assets.response(&requested).await
}

#[cfg(test)]
mod tests {
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;
    use crate::index::tests::Dir;

    #[tokio::test]
    async fn serves_fingerprinted_files_for_good() {
        let dir = Dir::new(&[("app.js", "console.log(1)")]);
        let assets = Arc::new(Assets::builder().dir(&dir.0).build().unwrap());
        let app: Router = Assets::router(assets.clone());

        let get = |uri: String| {
            let app = app.clone();
            async move {
                app.oneshot(Request::get(uri).body(Body::empty()).unwrap())
                    .await
                    .unwrap()
            }
        };
        let response = get(assets.url("app.js")).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "text/javascript");
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "public, max-age=31536000, immutable"
        );
        assert_eq!(
            response.into_body().collect().await.unwrap().to_bytes(),
            "console.log(1)"
        );

        assert_eq!(
            get("/assets/app.js".into()).await.headers()[header::CACHE_CONTROL],
            "no-cache"
        );
        assert_eq!(get("/assets/nope.js".into()).await.status(), StatusCode::NOT_FOUND);
    }
}
