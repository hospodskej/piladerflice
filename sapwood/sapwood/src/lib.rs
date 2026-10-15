//! Sapwood: a small web framework for server-rendered sites on
//! [Axum](https://docs.rs/axum).
//!
//! It is the part of a site that isn't about your business: the visitor's
//! session, forms that can't be forged, texts in several languages, and
//! static files that browsers may cache forever. Your pages stay plain Axum
//! handlers rendering HTML (with MiniJinja, or anything else).
//!
//! | Module | Crate | What it does |
//! |--------|-------|--------------|
//! | [`session`] | `sapwood-session` | encrypted cookie sessions with your own data type, CSRF token, flash messages, login cookie |
//! | [`forms`] | `sapwood-forms` | form extractors that check the CSRF token first (plain, typed, uploads), validation errors |
//! | [`i18n`] | `sapwood-i18n` | YAML translations, `{name}` placeholders, a fallback language, a MiniJinja `t()` |
//! | [`assets`] | `sapwood-assets` | fingerprinted static files, a JavaScript import map, an Axum router |
//!
//! Each is also its own crate, for sites that want only one; this crate is
//! all of them as one dependency (turn parts off with `default-features =
//! false` and the features `session`, `forms`, `i18n`, `assets`).
//!
//! ```no_run
//! use axum::{Router, routing::{get, post}, response::{Html, Redirect}};
//! use sapwood::forms::{Errors, Form};
//! use sapwood::session::{Flash, Session, SessionLayer};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Clone, Default, Serialize, Deserialize)]
//! struct Visit { name: Option<String> }
//!
//! #[derive(Deserialize)]
//! struct NameForm { name: String }
//!
//! async fn page(session: Session<Visit>) -> Html<String> {
//!     let name = session.read(|visit| visit.name.clone()).unwrap_or("stranger".into());
//!     let flash = session.take_flash().map(|flash| format!("{flash:?}")).unwrap_or_default();
//!     Html(format!(
//!         r#"<p>{flash}</p><p>Hello, {name}!</p>
//!            <form method="post"><input type="hidden" name="csrf_token" value="{}">
//!            <input name="name"><button>Save</button></form>"#,
//!         session.csrf_token()
//!     ))
//! }
//!
//! async fn save(session: Session<Visit>, Form(form): Form<NameForm>) -> Redirect {
//!     let mut errors = Errors::new();
//!     if form.name.trim().is_empty() {
//!         errors.add("name", "Please enter a name");
//!     }
//!     match errors.result(form.name) {
//!         Ok(name) => {
//!             session.update(|visit| visit.name = Some(name));
//!             session.set_flash(Flash::Notice("Saved".into()));
//!         }
//!         Err(errors) => session.set_flash(Flash::Alert(errors.messages().join(", "))),
//!     }
//!     Redirect::to("/")
//! }
//!
//! # async fn run() {
//! let secret = std::env::var("SECRET_KEY_BASE").unwrap(); // 64 random bytes or more
//! let app: Router = Router::new()
//!     .route("/", get(page).post(save))
//!     .layer(SessionLayer::<Visit>::new(secret.as_bytes()));
//! let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
//! axum::serve(listener, app).await.unwrap();
//! # }
//! ```
//!
//! A complete small app is in `examples/guestbook.rs` (`cargo run --example
//! guestbook`). Sapwood grew out of a real shop, [Pila
//! Derflice](https://github.com/hospodskej/piladerflice), which uses all of it.

#[cfg(feature = "assets")]
pub use sapwood_assets as assets;
#[cfg(feature = "forms")]
pub use sapwood_forms as forms;
#[cfg(feature = "i18n")]
pub use sapwood_i18n as i18n;
#[cfg(feature = "session")]
pub use sapwood_session as session;
