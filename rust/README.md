# Pila Derflice in Rust

The Rust version of the site, built step by step next to the Rails app. Rails stays the live site until this version does everything Rails does; then it takes over.

It shares the Rails app's files instead of copying them, so both always look the same:

- translations: `../config/locales/cs.yml`, `de.yml`
- images and CSS: `../app/assets/`
- JavaScript: `../app/javascript/` (Turbo and Stimulus are in `vendor/javascript/`)

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

The reverse proxy should pass `Host` and `X-Forwarded-Proto` so share links and the sitemap use the right `https://` address.

## Progress

| Phase | What | State |
|-------|------|-------|
| 1 | Server, layout (header, footer, menu, cookie banner, breadcrumbs, SEO and share tags), cs/de with `/at` URLs, legal pages, Sortiment pages, 404, robots.txt, health check | done |
| 2 | Catalogue: home page, e-shop, products, firewood grid, price list, services, euro prices, sitemap (needs the database) | next |
| 3 | Cart and checkout, delivery price, order emails | |
| 4 | Accounts: registration, login, profile pictures | |
| 5 | Admin area | |
| 6 | Remaining extras, side-by-side comparison with Rails, switch-over | |

Phase 1 output was compared with Rails page by page: the HTML is the same apart from asset fingerprints, and browser screenshots are pixel-identical.

## How the code is organised

```
src/
  main.rs        starts the server (reads HOST, PORT, APP_ENV)
  lib.rs         AppState (shared by all requests) and app(): the whole site as one service
  request.rs     takes /at off the URL before routing; Req = language, paths, site address
  locale.rs      Czech/German and their URLs (port of Rails' LocalizedPath)
  i18n.rs        reads the YAML translations; t("key", name=value) like Rails I18n
  assets.rs      fingerprinted static files (like Propshaft) and the JavaScript import map
  views.rs       templates, page title/description, breadcrumbs, share tags
  routes/
    mod.rs       the list of all URLs, and the old ?locale= redirect
    pages.rs     legal and Sortiment pages
    errors.rs    the 404 page with a random mascot
    system.rs    /up and /robots.txt
templates/       Jinja templates (MiniJinja); same HTML as the Rails views
tests/site.rs    requests against the whole app, like a browser would make them
```

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
- **Tests next to the code**: `#[cfg(test)] mod tests` at the bottom of a file; whole-app tests in `tests/`.

Good next reads: *The Rust Programming Language* (https://doc.rust-lang.org/book/), chapters 4 (ownership), 6 (enums), 9 (errors) and 10 (traits), then the axum examples (https://github.com/tokio-rs/axum/tree/main/examples).
