//! The islands' WebAssembly (`islands/`, built by `cargo leptos build`):
//! the module and its JavaScript loader, served at `/pkg/<version>/...`.
//! The version is a fingerprint of both files, so browsers keep them for
//! good and a new build simply gets new URLs.
//!
//! Without a build (plain `cargo build`, the tests) the pages work as
//! plain HTML: [`Wasm::pkg_path`] is None and no script is added.

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use axum::Router;
use axum::body::Body;
use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use tower_http::compression::CompressionLayer;

/// The name cargo-leptos gives the files (`output-name` in Cargo.toml).
pub const OUTPUT_NAME: &str = "piladerflice";

pub struct Wasm {
    dir: PathBuf,
    /// Development: notice a new build on the next page view.
    watch: bool,
    current: RwLock<Option<Build>>,
}

#[derive(Clone)]
struct Build {
    version: String,
    modified: Vec<SystemTime>,
}

impl Wasm {
    pub fn load(dir: impl Into<PathBuf>, watch: bool) -> Self {
        let dir = dir.into();
        let current = RwLock::new(Build::read(&dir));
        if current.read().unwrap().is_none() {
            tracing::warn!(
                "no WebAssembly build in {}: the pages work without islands (run `cargo leptos build`)",
                dir.display()
            );
        }
        Self { dir, watch, current }
    }

    /// Where the files are served from, e.g. "pkg/1a2b3c4d"; None without a
    /// build.
    pub fn pkg_path(&self) -> Option<String> {
        if self.watch {
            let changed = self
                .current
                .read()
                .unwrap()
                .as_ref()
                .map(|build| build.modified.clone())
                != Some(modified(&self.dir));
            if changed {
                *self.current.write().unwrap() = Build::read(&self.dir);
            }
        }
        let version = self.current.read().unwrap().as_ref()?.version.clone();
        Some(format!("pkg/{version}"))
    }

    pub fn router<S: Clone + Send + Sync + 'static>(wasm: Arc<Wasm>) -> Router<S> {
        Router::new()
            .route("/pkg/{version}/{file}", get(serve))
            .layer(CompressionLayer::new())
            .with_state(wasm)
    }
}

impl Build {
    fn read(dir: &Path) -> Option<Build> {
        let mut content = std::fs::read(dir.join(format!("{OUTPUT_NAME}.js"))).ok()?;
        content.extend(std::fs::read(wasm_file(dir)?).ok()?);
        let fingerprint = sapwood_assets::fingerprinted_path("v", &content);
        Some(Build {
            version: fingerprint.trim_start_matches("v-").to_string(),
            modified: modified(dir),
        })
    }
}

/// cargo-leptos names the module `piladerflice.wasm`; wasm-bindgen alone,
/// `piladerflice_bg.wasm`.
fn wasm_file(dir: &Path) -> Option<PathBuf> {
    [format!("{OUTPUT_NAME}.wasm"), format!("{OUTPUT_NAME}_bg.wasm")]
        .into_iter()
        .map(|name| dir.join(name))
        .find(|path| path.exists())
}

fn modified(dir: &Path) -> Vec<SystemTime> {
    let js = dir.join(format!("{OUTPUT_NAME}.js"));
    [Some(js), wasm_file(dir)]
        .into_iter()
        .flatten()
        .filter_map(|path| std::fs::metadata(path).and_then(|meta| meta.modified()).ok())
        .collect()
}

async fn serve(State(wasm): State<Arc<Wasm>>, UrlPath((version, file)): UrlPath<(String, String)>) -> Response {
    if wasm.pkg_path() != Some(format!("pkg/{version}")) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let (path, mime) = if file == format!("{OUTPUT_NAME}.js") {
        (Some(wasm.dir.join(&file)), "text/javascript")
    } else if file == format!("{OUTPUT_NAME}.wasm") || file == format!("{OUTPUT_NAME}_bg.wasm") {
        (wasm_file(&wasm.dir), "application/wasm")
    } else {
        (None, "")
    };
    let Some(bytes) = path.and_then(|path| std::fs::read(path).ok()) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mut response = Response::new(Body::from(bytes));
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(mime));
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    response
}

#[cfg(test)]
mod tests {
    use axum::http::Request;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn serves_the_build_under_its_version_only() {
        let dir = std::env::temp_dir().join(format!("wasm-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(Wasm::load(&dir, false).pkg_path().is_none(), "no build yet");

        std::fs::write(dir.join("piladerflice.js"), "export default 1").unwrap();
        std::fs::write(dir.join("piladerflice.wasm"), b"\0asm").unwrap();
        let wasm = Arc::new(Wasm::load(&dir, false));
        let pkg = wasm.pkg_path().unwrap();
        let app: Router = Wasm::router(wasm);
        let status = |uri: String| {
            let app = app.clone();
            async move {
                app.oneshot(Request::get(uri).body(Body::empty()).unwrap())
                    .await
                    .unwrap()
                    .status()
            }
        };
        assert_eq!(status(format!("/{pkg}/piladerflice.js")).await, StatusCode::OK);
        assert_eq!(status(format!("/{pkg}/piladerflice.wasm")).await, StatusCode::OK);
        assert_eq!(status(format!("/{pkg}/piladerflice_bg.wasm")).await, StatusCode::OK);
        assert_eq!(status(format!("/{pkg}/other.js")).await, StatusCode::NOT_FOUND);
        assert_eq!(
            status("/pkg/00000000/piladerflice.js".into()).await,
            StatusCode::NOT_FOUND
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
