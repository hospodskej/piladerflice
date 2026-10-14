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
deploy/package.sh         # -> dist/pila-derflice-<commit>.tar.gz
```

The release is a folder: the program (15 MB, needs only the C library) and the files it reads, laid out like this repository: `rust/templates`, `config/locales`, `app/assets` (the site's images are most of its 36 MB), `app/javascript`, `public/`, plus `deploy/`. On the server it runs from that folder with `APP_ROOT` pointing at it. **`deploy/SWITCH-OVER.md`** is the step-by-step guide: setting up the server (systemd, Caddy for HTTPS, nightly backups), putting a release on it, moving the live site from Rails, the checklist afterwards, and going back if needed.

`piladerflice backup <file>` copies the database to a new file while the site keeps running (the nightly backup uses it).

Settings come from environment variables:

| Variable   | Default     | What it does |
|------------|-------------|--------------|
| `HOST`     | `127.0.0.1` | Address to listen on. Keep it local and put nginx or Caddy in front for HTTPS. |
| `PORT`     | `4000`      | Port to listen on. |
| `APP_ENV`  | `development` in debug builds, `production` in release builds | `production` loads templates and assets once at startup. |
| `RUST_LOG` | `info`      | Log detail, e.g. `debug` or `piladerflice=debug`. |
| `APP_ROOT` | this repository | The release folder on a server (templates, translations, assets, `public/`). Then the database defaults to `APP_ROOT/storage/production.sqlite3`. |
| `DATABASE_URL` | the Rails development database | The SQLite database, e.g. `sqlite:/srv/pila/production.sqlite3`. |
| `STORAGE_DIR` | `../storage` | Where the uploaded images live (Active Storage's folder). |
| `SECRET_KEY_BASE` | a fixed development value | Encrypts the session and login cookies. **Required in production**, at least 64 characters (e.g. `openssl rand -hex 64`). Changing it signs everyone out and empties their carts. |
| `ORS_API_KEY` | none | openrouteservice key for the delivery price. Without it, delivery is priced individually. |
| `SMTP_USERNAME`, `SMTP_PASSWORD`, `SMTP_ADDRESS`, `SMTP_PORT` | none, smtp.gmail.com, 587 | The mail server. Without `SMTP_USERNAME`, e-mails are only logged. Port 465 uses TLS from the start, other ports STARTTLS. |
| `MAIL_FROM_ADDRESS`, `ORDER_NOTIFICATION_EMAIL`, `INQUIRY_NOTIFICATION_EMAIL` | objednavky@piladerflice.cz, … | Sender, and who gets told about new orders and quote requests. |

The reverse proxy should pass `Host` and `X-Forwarded-Proto` so share links and the sitemap use the right `https://` address.

## Progress

| Phase | What | State |
|-------|------|-------|
| 1 | Server, layout (header, footer, menu, cookie banner, breadcrumbs, SEO and share tags), cs/de with `/at` URLs, legal pages, Sortiment pages, 404, robots.txt, health check | done |
| 2 | Catalogue: home page, e-shop, product pages (firewood grid, lumber filters, simple variants), contact page with price list and quote form, services, Sortiment, euro prices, sitemap, uploaded images | done |
| 3 | Sessions (encrypted cookie) and form protection, cart with Turbo Streams, checkout (shipping, details, summary, thank-you), delivery price by distance, order and quote request e-mails | done |
| 4 | Accounts: registration, sign-in and sign-out, profile, profile pictures, order history and reordering, checkout prefill | done |
| 5 | Admin area: overview, orders, quote requests, catalogue products and variants with photos, promos, price list, quote form choices | done |
| 6 | Czech time; the security headers, `public/` files and old image addresses of Rails; startup and shutdown safety; backups; a comparison of every page; the release package and the switch-over guide | done |

Every page so far was compared with Rails: the HTML is the same apart from asset fingerprints, form tokens and uploaded-image URLs, and browser screenshots are pixel-identical. The whole shopping flow (add to cart, change quantities, all checkout steps, the thank-you page) was run in a browser against both apps, in Czech and German, and every step's HTML matched; the saved orders and quote requests are the same too. The account flow (sign-in required, registration with errors, profile, picture upload, checkout prefill, sign-out, failed and successful sign-in) was compared the same way, in both languages.

Before the switch-over, every page of the sitemap (Czech and German) and a set of odd addresses were fetched from both apps once more: same status, same HTML. The packaged release was also run from an empty folder with production settings and served the same pages.

The admin was compared the same way: all 29 admin pages (including forms that didn't save) have the same HTML as Rails, and a script that adds, edits, moves and deletes one of everything (with a photo upload) gave the same page at every step and, on two copies of the same database, the same data afterwards.

Accounts are shared with Rails: passwords are bcrypt hashes in the same format as Rails' `has_secure_password`, and sign-ins are rows of the same `sessions` table, so an account made in one app signs in to the other (tested both ways). Profile pictures are stored as Active Storage blobs, cleaned the same way (decoded, turned upright, cut to 256×256, saved as WebP).

A few things differ from Rails on purpose:

- order errors read "Při nákupu na firmu je potřeba vyplnit název firmy" without Rails' English field name in front;
- error counts use the Czech plural ("2 chyby", Rails says "2 chyba");
- the session cookie has a new name (`_pila_session`) and the login cookie is `_pila_login`, so switching from Rails to Rust starts everyone with an empty cart and signed out;
- a sign-in with an unknown e-mail takes as long as one with a wrong password, so the timing doesn't tell which e-mails have accounts;
- picture uploads are limited per account (10 per 10 minutes) instead of per IP address;
- photos uploaded in the admin must be JPEG, PNG or WebP (Rails took any file). Their metadata (GPS position, camera, date) is removed as in Rails, but without recompressing them, so they are stored a little bigger, and a rotated phone photo keeps its orientation (ImageOptim drops it, so Rails shows such photos lying on their side).

Smaller differences: Rails sends a `Link` header preloading the stylesheets and an `X-Request-Id`, Rust doesn't; Rails' old image addresses are redirected (308) instead of served; `robots.txt` has a reworded comment.

The sign-in, sign-out and registration messages are in Czech in both languages, as in Rails; translating them is a small fix for later, in both apps.

The comparison found bugs in the Rails app too, fixed in both: the admin's error messages read "Translation missing" (no Czech validation messages), "2 chyba" instead of "2 chyby", an emptied "Pořadí" field crashed saving a product or variant, and old orders showed "Translation missing" for products deleted since. Both apps now show times in Czech time (`czech_time.rs`; Rails: `config.time_zone = "Prague"`), still storing UTC.

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
  forms/             submitted forms: checks the form protection token, then reads the fields
                     (upload.rs: the same for forms with a file)
  session/           the encrypted session cookie: data, load/save layer, CSRF token, flash
  shop/              cart and cart lines, checkout details, line descriptions, VAT,
    delivery/        the delivery quote and the openrouteservice distance lookup
  admin/             what the admin edits: one file per table (columns, Rails' checks),
                     record.rs (a row as form values and back), images.rs (checking and
                     cleaning uploaded photos), ordering.rs, dashboard.rs
  czech_time.rs      Czech time (UTC+1, summer UTC+2) for showing dates and counting days
  db.rs              the connection, the schema version check, backups
  storage.rs         files the way Active Storage keeps them (blob, attachment, ab/cd/key)
  accounts/          the signed-in user, passwords (bcrypt), registration checks, rate limits
    avatar/          profile pictures: storing as Active Storage blobs, cleaning uploads,
                     keeping the EXIF data inside the WebP
  mail/              e-mails: SMTP or outbox, settings, order and quote request mails
  assets/            static files: index + fingerprints, import map, /assets handler
  money/             Kč and € formatting; the live exchange rate
  models/            one file per table
    catalog/         product, variant, firewood grid
    content/         promo, service, category tile, FAQ slide, review
    user.rs          accounts and their profile; login_session.rs: the sessions table
    validation.rs    Rails-style error messages ("Klíč už je použitý") from the translations
    order.rs, inquiry.rs, pricelist.rs, inquiry_options.rs, upload.rs
  views/             templates and the layout
    templates.rs     loading templates; functions.rs: t() and path()
    layout.rs        renders a page inside the site layout; admin_layout.rs: the admin's
    form_helpers.rs  Rails' form helpers (text_field, select, button_to...) for the admin
    page.rs          title, description, breadcrumbs, status of a page
    breadcrumbs.rs, seo.rs, text.rs
  routes/            one file per page; mod.rs is the list of all URLs
    home.rs, sluzby.rs, sortiment.rs, legal.rs, product_range.rs
    kontakt/         the page, its price list, its quote form
    cart/            cart page, add, change quantity/remove, Turbo Streams
    checkout/        one file per step, the guard between steps, the order draft
    inquiries.rs     the quote request form
    auth/            sign-in, sign-out, registration
    account/         the account page, profile form, profile picture, ordering again
    admin/           the admin: guard.rs lets only admins in, params.rs reads record ids,
                     one file per section
    eshop/           listing, product page, one file per product layout
                     (firewood, lumber, simple_variant), recommendations
    components/      view pieces used by several pages (product card,
                     category tile, business data for search engines, text formatting)
    error.rs         what a failing request shows (views/error_pages.rs: public/500.html)
    headers.rs       the protective headers on every response
    public_files.rs  /icon.png and the other files of public/
    legacy_storage.rs  Rails' /rails/active_storage image addresses, sent on to /uploads
    not_found.rs, legacy_locale.rs, system.rs, sitemap.rs, uploads.rs
deploy/              release package script, systemd units, Caddyfile, backups, SWITCH-OVER.md
templates/           Jinja templates (MiniJinja); same HTML as the Rails views (mail/: the e-mails)
db/schema.sql        the database schema (exported from Rails), used by the tests
tests/               requests against the whole app, like a browser would make them;
                     fixtures/data.sql is the test data, loaded into a fresh in-memory database;
                     fixtures/accounts.sql adds two accounts (password "heslo12345"), one an admin
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
- **Shared, changeable state** (`session/mod.rs`): `Arc<Mutex<…>>` lets the session layer and the handler see the same data; `read` and `update` keep the lock short.
- **Extractors** (`forms.rs`): `impl FromRequest for FormData` means a handler that takes `form: FormData` can only run after the token check passed.
- **`Send` futures**: axum runs handlers on any thread, so nothing that isn't `Send` may live across an `.await`. `shop/delivery/route_service.rs` shows the fix for one such error (finish the URL builder into a `String` first); `#[axum::debug_handler]` explains these errors best.
- **`spawn_blocking`** (`routes/account/avatar.rs`): resizing a picture is CPU work that would stall the async threads, so it runs on a thread meant for blocking work.
- **`Box`** (`require_login` in `routes/account/mod.rs`): puts a big value on the heap so the `Result` that carries it stays small; clippy points these out.
- **Middleware** (`routes/admin/guard.rs`): a function that runs before every admin route; it answers itself (a redirect) or passes the request on with `next.run(...)`. Checking access in one place means no page can forget it.
- **Custom extractors with their own rejection** (`routes/admin/params.rs`): `RecordId` reads `{id}` from the URL and answers "not found" by itself when it isn't a number.
- **`const` tables** (`admin/products.rs`): each admin table's columns are a constant, checked by the compiler and by a test that every column can be changed by its form.
- **Byte-level parsing** (`admin/images.rs`): walking a JPEG's or PNG's blocks with slices and `get(..)?`, which stops at a broken file instead of panicking.
- **Graceful shutdown** (`main.rs`): `tokio::select!` waits for whichever comes first, Ctrl+C or systemd's SIGTERM; the server then finishes the requests it is answering.
- **`OnceLock`** (`views/error_pages.rs`): a value set once at startup and read from anywhere afterwards, safely across threads.
- **Tests next to the code**: `#[cfg(test)] mod tests` at the bottom of a file; whole-app tests in `tests/`.

Good next reads: *The Rust Programming Language* (https://doc.rust-lang.org/book/), chapters 4 (ownership), 6 (enums), 9 (errors) and 10 (traits), then the axum examples (https://github.com/tokio-rs/axum/tree/main/examples).
