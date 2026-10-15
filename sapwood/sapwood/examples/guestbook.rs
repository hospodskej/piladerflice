//! A guestbook: a form protected against forgery, messages kept in memory,
//! a flash message after posting, and the page in English or Czech.
//!
//!     cargo run --example guestbook
//!
//! then open http://127.0.0.1:3000 (or /?lang=cs).

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::{Query, State};
use axum::response::{Html, Redirect};
use axum::routing::get;
use minijinja::{Environment, context};
use sapwood::forms::{Errors, Form};
use sapwood::i18n::{Translations, jinja};
use sapwood::session::{Flash, Session, SessionLayer};
use serde::{Deserialize, Serialize};

const EN: &str = r#"
title: Guestbook
sign: Sign the guestbook
name: Your name
message: Message
send: Send
thanks: "Thank you, {name}!"
missing: Please fill in your name and a message.
"#;

const CS: &str = r#"
title: Kniha návštěv
sign: Zapište se
name: Vaše jméno
message: Vzkaz
send: Odeslat
thanks: "Děkujeme, {name}!"
missing: Vyplňte prosím jméno a vzkaz.
"#;

const PAGE: &str = r#"<!doctype html>
<html lang="{{ locale }}"><head><meta charset="utf-8"><title>{{ t("title") }}</title></head>
<body>
  <h1>{{ t("title") }}</h1>
  {% if flash %}<p class="{{ flash.kind }}">{{ flash.text }}</p>{% endif %}
  {% for entry in entries %}<blockquote>{{ entry.message }} — {{ entry.name }}</blockquote>{% endfor %}
  <h2>{{ t("sign") }}</h2>
  <form method="post" action="/?lang={{ locale }}">
    <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
    <label>{{ t("name") }} <input name="name"></label>
    <label>{{ t("message") }} <textarea name="message"></textarea></label>
    <button>{{ t("send") }}</button>
  </form>
</body></html>"#;

/// What the session remembers: nothing of our own here, only the
/// framework's token and flash message.
type Visit = ();

#[derive(Clone, Serialize)]
struct Entry {
    name: String,
    message: String,
}

#[derive(Deserialize)]
struct EntryForm {
    name: String,
    message: String,
}

#[derive(Deserialize)]
struct Lang {
    lang: Option<String>,
}

#[derive(Clone)]
struct App {
    templates: Arc<Environment<'static>>,
    translations: Arc<Translations>,
    entries: Arc<Mutex<Vec<Entry>>>,
}

impl App {
    fn locale(lang: &Lang) -> &'static str {
        if lang.lang.as_deref() == Some("cs") { "cs" } else { "en" }
    }
}

async fn show(State(app): State<App>, session: Session<Visit>, Query(lang): Query<Lang>) -> Html<String> {
    let entries = app.entries.lock().unwrap().clone();
    let html = app
        .templates
        .get_template("page.html")
        .and_then(|page| {
            page.render(context! {
                locale => App::locale(&lang),
                entries => entries,
                flash => session.take_flash(),
                csrf_token => session.csrf_token(),
            })
        })
        .unwrap_or_else(|error| format!("template error: {error}"));
    Html(html)
}

async fn sign(
    State(app): State<App>,
    session: Session<Visit>,
    Query(lang): Query<Lang>,
    Form(form): Form<EntryForm>,
) -> Redirect {
    let locale = App::locale(&lang);
    let mut errors = Errors::new();
    if form.name.trim().is_empty() || form.message.trim().is_empty() {
        errors.add("", app.translations.text(locale, "missing", &[]));
    }
    match errors.result(form) {
        Ok(form) => {
            let thanks = app.translations.text(locale, "thanks", &[("name", form.name.trim())]);
            app.entries.lock().unwrap().push(Entry {
                name: form.name.trim().to_string(),
                message: form.message.trim().to_string(),
            });
            session.set_flash(Flash::Notice(thanks));
        }
        Err(errors) => session.set_flash(Flash::Alert(errors.messages().join(" "))),
    }
    Redirect::to(&format!("/?lang={locale}"))
}

#[tokio::main]
async fn main() {
    let translations = Arc::new(
        Translations::from_sources([("en", EN), ("cs", CS)])
            .unwrap()
            .with_fallback("en"),
    );
    let mut templates = Environment::new();
    templates.add_function("t", jinja::function(translations.clone(), "locale"));
    templates.add_template("page.html", PAGE).unwrap();
    let app = App {
        templates: Arc::new(templates),
        translations,
        entries: Arc::default(),
    };

    // A real site reads the secret from its settings: 64 random bytes.
    let secret = std::env::var("SECRET_KEY_BASE").unwrap_or_else(|_| "x".repeat(64));
    let router = Router::new()
        .route("/", get(show).post(sign))
        .with_state(app)
        .layer(SessionLayer::<Visit>::new(secret.as_bytes()).secure(false));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("guestbook on http://127.0.0.1:3000");
    axum::serve(listener, router).await.unwrap();
}
