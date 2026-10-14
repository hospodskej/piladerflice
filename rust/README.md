# Pila Derflice in Rust

The Rust version of the site, built step by step next to the Rails app. Rails stays the live site until this version does everything Rails does; then it takes over.

It shares the Rails app's files instead of copying them, so both always look the same:

- translations: `../config/locales/cs.yml`, `de.yml`
- images and CSS: `../app/assets/`
- JavaScript: `../app/javascript/` (Turbo and Stimulus are in `vendor/javascript/`)
- the database: `../storage/development.sqlite3`, and uploaded images in `../storage/`

## Running it

You need Rust (https://rustup.rs). Then, in this folder:

```sh
cargo run                 # development server on http://127.0.0.1:4000
cargo test                # all tests
cargo clippy --all-targets  # lint
cargo fmt                 # format the code
```

In development, edits to templates, CSS, JavaScript and images show up on the next page load without a restart. Edits to `.rs` files need a new `cargo run`.

### Production

```sh
cargo build --release     # -> target/release/piladerflice (one ~6 MB file)
APP_ENV=production PORT=4000 ./target/release/piladerflice
```

Settings come from environment variables:

| Variable   | Default     | What it does |
|------------|-------------|--------------|
| `HOST`     | `127.0.0.1` | Address to listen on. Keep it local and put nginx or Caddy in front for HTTPS. |
| `PORT`     | `4000`      | Port to listen on. |
| `APP_ENV`  | `development` in debug builds, `production` in release builds | `production` loads templates and assets once at startup. |
| `RUST_LOG` | `info`      | Log detail, e.g. `debug` or `piladerflice=debug`. |
| `DATABASE_URL` | the Rails development database | The SQLite database, e.g. `sqlite:/srv/pila/production.sqlite3`. |
| `STORAGE_DIR` | `../storage` | Where the uploaded images live (Active Storage's folder). |

The reverse proxy should pass `Host` and `X-Forwarded-Proto` so share links and the sitemap use the right `https://` address.

## Progress

| Phase | What | State |
|-------|------|-------|
| 1 | Server, layout (header, footer, menu, cookie banner, breadcrumbs, SEO and share tags), cs/de with `/at` URLs, legal pages, Sortiment pages, 404, robots.txt, health check | done |
| 2 | Catalogue: home page, e-shop, product pages (firewood grid, lumber filters, simple variants), contact page with price list and quote form, services, Sortiment, euro prices, sitemap, uploaded images | done |
| 3 | Sessions and form protection, cart and checkout, delivery price, quote requests, order emails | next |
| 4 | Accounts: registration, login, profile pictures | |
| 5 | Admin area | |
| 6 | Remaining extras, side-by-side comparison with Rails, switch-over | |

Every page so far was compared with Rails: the HTML is the same apart from asset fingerprints, form tokens and uploaded-image URLs, and browser screenshots are pixel-identical.

Uploaded images are served at `/uploads/<key>/<file name>` instead of Rails' signed `/rails/active_storage/...` links. The key is random and never reused, so these URLs are cached for good.

## How the code is organised

Small files, one job each: a page, a table, a piece of the layout. A file's first lines (`//!`) say what it is for. Folders group what belongs together; their `mod.rs` lists the files and says how they fit.

```
src/
  main.rs            starts the server (reads HOST, PORT, APP_ENV, DATABASE_URL...)
  lib.rs             app(): the whole site as one service
  config.rs          settings: where files, database and uploads are
  state.rs           AppState, shared by all requests
  request.rs         takes /at off the URL before routing; Req = language, paths, site address
  locale.rs          Czech/German and their URLs (port of Rails' LocalizedPath)
  i18n.rs            the YAML translations; t("key", name=value) like Rails I18n
  db.rs              the SQLite connection
  assets/            static files: index + fingerprints, import map, /assets handler
  money/             Kč and € formatting; the live exchange rate
  models/            one file per table
    catalog/         product, variant, firewood grid
    content/         promo, service, category tile, FAQ slide, review
    pricelist.rs, inquiry_options.rs, upload.rs
  views/             templates and the layout
    templates.rs     loading templates; functions.rs: t() and path()
    layout.rs        renders a page inside the site layout
    page.rs          title, description, breadcrumbs, status of a page
    breadcrumbs.rs, seo.rs, text.rs
  routes/            one file per page; mod.rs is the list of all URLs
    home.rs, sluzby.rs, sortiment.rs, legal.rs, product_range.rs
    kontakt/         the page, its price list, its quote form
    eshop/           listing, product page, one file per product layout
                     (firewood, lumber, simple_variant), recommendations
    components/      view pieces used by several pages (product card,
                     category tile, business data for search engines, text formatting)
    error.rs         what a failing request shows
    not_found.rs, legacy_locale.rs, system.rs, sitemap.rs, uploads.rs
templates/           Jinja templates (MiniJinja); same HTML as the Rails views
db/schema.sql        the database schema (exported from Rails), used by the tests
tests/               requests against the whole app, like a browser would make them;
                     fixtures/data.sql is the test data, loaded into a fresh in-memory database
```

When a Rails migration changes the database, re-export `db/schema.sql` (the `CREATE` statements from `sqlite_master`).

### Templates

Templates use Jinja syntax, which reads much like ERB:

| Rails (ERB) | Rust (Jinja) |
|-------------|--------------|
| `<%= t("nav.domu") %>` | `{{ t("nav.domu") }}` |
| `<%= t("checkout.delivery_line", km: 100) %>` | `{{ t("checkout.delivery_line", km=100) }}` |
| `localized_path("/kontakt#cenik")` | `path("/kontakt#cenik")` |
| `image_tag "logo-wm.webp"` | `<img src="{{ asset('logo-wm.webp') }}">` |
| `<% items.each do \|item\| %>` … `<% end %>` | `{% for item in items %}` … `{% endfor %}` |
| `render "shared/footer"` | `{% include "shared/footer.html" %}` |
| layout + `yield` | `{% extends "layouts/application.html" %}` + `{% block content %}` |

Text is HTML-escaped automatically; translation keys ending in `_html` are trusted HTML, as in Rails.

### Adding a page

1. Write the template in `templates/`, starting with `{% extends "layouts/application.html" %}`.
2. Add a handler (a function that answers the request) in `src/routes/`. It builds a `Page` with the title and description and calls `views::render`.
3. Add the URL in `src/routes/mod.rs`. It works under `/at` automatically.
4. Add a test in `tests/site.rs`.

## Rust concepts you'll meet in this code

- **Ownership and borrowing** (`&str` vs `String`, `&AppState`): who owns a value and who only looks at it. The compiler checks this, so there are no use-after-free bugs and no data races.
- **`Option` and `Result`** instead of `nil` and exceptions. `?` means "return the error to my caller". See `Translations::load`.
- **Enums with `match`**: `Locale` in `locale.rs`. The compiler makes sure every language is handled.
- **Traits**: `FromRequestParts` for `Req` in `request.rs` lets a handler simply ask for `req: Req`, and axum builds it from the request.
- **`async`/`await`**: handlers are async so one small process can serve many visitors at once (tokio is the runtime).
- **`Arc`**: shared ownership across threads. `AppState` wraps everything in one `Arc`, so cloning it per request is just a counter bump.
- **`sqlx::FromRow`**: a struct per table row (`models/catalog.rs`); queries return typed values instead of hashes.
- **Lifetimes** (`FirewoodGrid<'a>`): the grid borrows the variants instead of copying them, and the compiler checks they live long enough. `routes/eshop/firewood.rs` has a small one: a closure returning an iterator needs `&'static str`.
- **Modules and privacy**: `mod` declares a file, `pub` makes things visible outside it, `pub use` re-exports. A child module may add methods to its parent's type (`assets/importmap.rs` adds `impl Assets`) and see its private fields.
- **Tests next to the code**: `#[cfg(test)] mod tests` at the bottom of a file; whole-app tests in `tests/`.

Good next reads: *The Rust Programming Language* (https://doc.rust-lang.org/book/), chapters 4 (ownership), 6 (enums), 9 (errors) and 10 (traits), then the axum examples (https://github.com/tokio-rs/axum/tree/main/examples).
