//! The middleware: loads the session from its cookies before the handler
//! runs, and sends new cookies with the response when it changed.

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::{HeaderValue, header};
use axum::response::Response;
use cookie::Key;
use cookie::time::Duration;
use tower_layer::Layer;
use tower_service::Service;

use crate::store::{self, CookieSettings};
use crate::{Session, SessionData};

/// Adds a [`Session<T>`] to every request. Configure it, then `.layer()` it
/// onto your router.
pub struct SessionLayer<T> {
    settings: Arc<CookieSettings>,
    _data: PhantomData<fn() -> T>,
}

impl<T> Clone for SessionLayer<T> {
    fn clone(&self) -> Self {
        Self {
            settings: self.settings.clone(),
            _data: PhantomData,
        }
    }
}

impl<T: SessionData> SessionLayer<T> {
    /// Sessions encrypted with a key derived from `secret`.
    ///
    /// # Panics
    ///
    /// If `secret` is shorter than 32 bytes; use 64 random bytes (e.g.
    /// `openssl rand -hex 64`).
    pub fn new(secret: &[u8]) -> Self {
        Self::with_key(Key::derive_from(secret))
    }

    pub fn with_key(key: Key) -> Self {
        Self {
            settings: Arc::new(CookieSettings {
                key,
                name: "_session".to_string(),
                secure: true,
                login: None,
            }),
            _data: PhantomData,
        }
    }

    /// The session cookie's name; `_session` by default.
    pub fn cookie_name(mut self, name: impl Into<String>) -> Self {
        self.settings_mut().name = name.into();
        self
    }

    /// Whether browsers may send the cookies over HTTPS only. On by default;
    /// turn it off for plain-HTTP development.
    pub fn secure(mut self, secure: bool) -> Self {
        self.settings_mut().secure = secure;
        self
    }

    /// Enables [`Session::set_login`]: a second cookie, `name`, lasting
    /// `lifetime`, so closing the browser doesn't sign anyone out.
    pub fn login_cookie(mut self, name: impl Into<String>, lifetime: std::time::Duration) -> Self {
        let lifetime = Duration::try_from(lifetime).unwrap_or(Duration::MAX);
        self.settings_mut().login = Some((name.into(), lifetime));
        self
    }

    fn settings_mut(&mut self) -> &mut CookieSettings {
        Arc::get_mut(&mut self.settings).expect("configure the SessionLayer before cloning it")
    }
}

impl<S, T> Layer<S> for SessionLayer<T> {
    type Service = SessionService<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionService {
            inner,
            settings: self.settings.clone(),
            _data: PhantomData,
        }
    }
}

/// The service [`SessionLayer`] wraps around your routes.
pub struct SessionService<S, T> {
    inner: S,
    settings: Arc<CookieSettings>,
    _data: PhantomData<fn() -> T>,
}

impl<S: Clone, T> Clone for SessionService<S, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            settings: self.settings.clone(),
            _data: PhantomData,
        }
    }
}

impl<S, T> Service<Request> for SessionService<S, T>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    T: SessionData,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        let settings = self.settings.clone();
        // The clone that was polled ready handles this request.
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move {
            let cookies = request
                .headers()
                .get(header::COOKIE)
                .and_then(|value| value.to_str().ok());
            let login = settings
                .login
                .as_ref()
                .and_then(|(name, _)| store::read(cookies, &settings.key, name));
            let session = Session::<T>::from_parts(store::load(cookies, &settings), login);
            request.extensions_mut().insert(session.clone());
            request.extensions_mut().insert(session.request_csrf());

            let mut response = inner.call(request).await?;
            for value in session.set_cookie_headers(&settings) {
                if let Ok(value) = HeaderValue::from_str(&value) {
                    response.headers_mut().append(header::SET_COOKIE, value);
                }
            }
            Ok(response)
        })
    }
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;
    use crate::Flash;

    #[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
    struct Count(u32);

    fn app() -> Router {
        Router::new()
            .route(
                "/count",
                get(|session: Session<Count>| async move {
                    session.update(|count| {
                        count.0 += 1;
                        count.0.to_string()
                    })
                }),
            )
            .route("/quiet", get(|| async { "nothing" }))
            .route(
                "/flash",
                get(|session: Session<Count>| async move {
                    session.set_flash(Flash::Notice("saved".into()));
                }),
            )
            .route(
                "/show",
                get(|session: Session<Count>| async move { format!("{:?}", session.take_flash()) }),
            )
            .route(
                "/login",
                get(|session: Session<Count>| async move { session.set_login(Some("42".into())) }),
            )
            .route(
                "/logout",
                get(|session: Session<Count>| async move { session.set_login(None) }),
            )
            .route(
                "/who",
                get(|session: Session<Count>| async move { format!("{:?}", session.login()) }),
            )
            .layer(
                SessionLayer::<Count>::new(&[1; 64])
                    .cookie_name("_test")
                    .secure(false)
                    .login_cookie("_login", std::time::Duration::from_secs(3600)),
            )
    }

    /// (body, Set-Cookie values) of a GET with these cookies.
    async fn request(path: &str, cookies: &str) -> (String, Vec<String>) {
        let request = Request::get(path)
            .header(header::COOKIE, cookies)
            .body(Body::empty())
            .unwrap();
        let response = app().oneshot(request).await.unwrap();
        let set: Vec<String> = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|value| value.to_str().unwrap().to_string())
            .collect();
        let body = String::from_utf8(response.into_body().collect().await.unwrap().to_bytes().to_vec()).unwrap();
        (body, set)
    }

    fn cookie_pair(set_cookie: &str) -> &str {
        set_cookie.split(';').next().unwrap()
    }

    #[tokio::test]
    async fn the_session_lasts_between_requests() {
        let (body, set) = request("/count", "").await;
        assert_eq!(body, "1");
        assert!(set[0].starts_with("_test="));
        let (body, _) = request("/count", cookie_pair(&set[0])).await;
        assert_eq!(body, "2");
    }

    #[tokio::test]
    async fn untouched_sessions_set_no_cookie() {
        let (_, set) = request("/quiet", "").await;
        assert!(set.is_empty());
    }

    #[tokio::test]
    async fn flash_shows_on_the_next_request_only() {
        let (_, set) = request("/flash", "").await;
        let (body, set) = request("/show", cookie_pair(&set[0])).await;
        assert_eq!(body, r#"Some(Notice("saved"))"#);
        let (body, _) = request("/show", cookie_pair(&set[0])).await;
        assert_eq!(body, "None");
    }

    #[tokio::test]
    async fn the_login_cookie_signs_in_and_out() {
        let (_, set) = request("/login", "").await;
        assert!(
            set[0].starts_with("_login=") && set[0].contains("Max-Age=3600"),
            "{}",
            set[0]
        );
        let (body, _) = request("/who", cookie_pair(&set[0])).await;
        assert_eq!(body, r#"Some("42")"#);
        let (_, set) = request("/logout", cookie_pair(&set[0])).await;
        assert!(set[0].starts_with("_login=;"));
        let (body, _) = request("/who", "_login=forged").await;
        assert_eq!(body, "None");
    }
}
