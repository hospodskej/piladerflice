# Pila Derflice

The website and e-shop of Pila Derflice (Štěpán Merta's sawmill): firewood, lumber and more, in Czech and, under `/at`, in German for Austrian customers, with euro prices. Customers order online or ask for a quote; the owner runs the catalogue, prices, promos and orders in the admin.

It is written in Rust, all of it: one small program that serves the pages, keeps its data in a single SQLite file and runs on any Linux server. The pages are [Leptos](https://leptos.dev) components rendered on the server; the interactive parts (the cart, the menus, the carousels, the quote form...) are Leptos *islands*, the same Rust compiled to WebAssembly, which the browser brings to life in place. There is no hand-written JavaScript. The web parts that aren't specific to the sawmill (sessions, forms, translations, static files) are **Sapwood**, a small framework in [`sapwood/`](sapwood/), meant to be published as crates.

## Running it

You need Rust (https://rustup.rs), the WebAssembly target and [cargo-leptos](https://github.com/leptos-rs/cargo-leptos), which builds the server and the islands together:

```sh
rustup target add wasm32-unknown-unknown    # once
cargo install cargo-leptos --locked         # once
```

Then:

```sh
cp .env.example .env      # and fill in what you need (admin account, openrouteservice key)
cargo run -- setup        # a new database with the starting content (storage/development.sqlite3)
cargo run -- admin        # the admin account from ADMIN_EMAIL / ADMIN_PASSWORD in .env
cargo leptos watch        # the site on http://127.0.0.1:4000, admin on /admin; rebuilt on every change
```

`cargo run` serves the site too, with the islands of the last `cargo leptos build` (target/site/pkg), or as plain HTML pages without them: the forms work either way.

```sh
cargo test                # all tests
cargo clippy --all-targets  # lint
cargo fmt                 # format the code
```

In development, edits to translations, CSS and images show up on the next page load without a restart. Edits to `.rs` files (the pages too) need a rebuild, which `cargo leptos watch` does by itself.

### Commands

| Command | What it does |
|---------|--------------|
| `piladerflice` | Serves the site. |
| `piladerflice setup` | Creates a new database with the tables, the starting content (price list, catalogue with pictures, promos, services, FAQ, reviews) and no accounts. Never touches an existing file. |
| `piladerflice migrate` | Applies waiting database migrations (`db/migrations/`, see the README there). The site refuses to start while one is waiting. |
| `piladerflice admin` | Creates the admin account from `ADMIN_EMAIL` and `ADMIN_PASSWORD`, or makes that account the admin with that password. |
| `piladerflice backup <file>` | Copies the database to a new file while the site keeps running. |

With cargo: `cargo run -- migrate` and so on.

### On a server

```sh
deploy/package.sh         # -> dist/pila-derflice-<commit>.tar.gz
```

The release is a folder: the program (needs only the C library), the islands' WebAssembly in `pkg/`, and the files the program reads, laid out like this repository (`locales`, `assets` — the site's images are most of the package — `public`, `db`, `deploy`). On the server it runs from that folder with `APP_ROOT` pointing at it. **`deploy/LAUNCH.md`** is the step-by-step guide: setting up the server (systemd, Caddy for HTTPS, nightly backups), putting a release on it, the first start, the checklist, and updates.

Settings come from environment variables (on a development machine also from `.env`):

| Variable   | Default     | What it does |
|------------|-------------|--------------|
| `HOST`     | `127.0.0.1` | Address to listen on. Keep it local and put Caddy (or nginx) in front for HTTPS. |
| `PORT`     | `4000`      | Port to listen on. |
| `APP_ENV`  | `development` in debug builds, `production` in release builds | `production` loads the assets and the WebAssembly once at startup and sends HSTS. |
| `RUST_LOG` | `info`      | Log detail, e.g. `debug` or `piladerflice=debug`. |
| `APP_ROOT` | this folder | The release folder on a server. Then the database defaults to `APP_ROOT/storage/production.sqlite3`. |
| `DATABASE_URL` | `storage/development.sqlite3` | The SQLite database, e.g. `sqlite:/var/lib/pila-derflice/production.sqlite3`. |
| `STORAGE_DIR` | `storage/uploads/` | Where uploaded images live. |
| `SECRET_KEY_BASE` | a fixed development value | Encrypts the session and login cookies. **Required in production**, at least 64 characters (`openssl rand -hex 64`). Changing it signs everyone out and empties their carts. |
| `ORS_API_KEY` | none | openrouteservice key for the delivery price. Without it, delivery is priced individually. |
| `SMTP_USERNAME`, `SMTP_PASSWORD`, `SMTP_ADDRESS`, `SMTP_PORT` | none, smtp.gmail.com, 587 | The mail server. Without `SMTP_USERNAME`, e-mails are only logged. Port 465 uses TLS from the start, other ports STARTTLS. |
| `MAIL_FROM_ADDRESS`, `ORDER_NOTIFICATION_EMAIL`, `INQUIRY_NOTIFICATION_EMAIL` | objednavky@piladerflice.cz, … | Sender, and who gets told about new orders and quote requests. |
| `ADMIN_EMAIL`, `ADMIN_PASSWORD` | none | Only for `piladerflice admin`. |

The reverse proxy should pass `Host` and `X-Forwarded-Proto` so share links and the sitemap use the right `https://` address.

## How the site works

**Languages and URLs.** Czech has no prefix, the (Austrian) German site lives under `/at` (`/at/kontakt`, `/at/eshop/tramy`). The language comes from the URL only, so a shared link opens in the language it was shared in. Each route is written once: `/at` is taken off before routing (`request.rs`), and components link with `path("/kontakt#cenik")`, which adds it back. German visitors see euro prices, converted at the day's ECB rate. The admin is Czech only.

**Delivery price.** Delivery to an address costs 40 Kč (incl. VAT) per km, counted there and back, one trip for the whole order. The road distance from the sawmill (Derflice 66) is looked up with [openrouteservice](https://openrouteservice.org) and cached for 30 days per address; the checkout shows the price as approximate and adds it to the total. Without `ORS_API_KEY`, or if the lookup fails, delivery is "priced individually" and the total is the goods only. The free plan (1,000 geocoding and 2,000 route requests a day) is plenty. Never commit the key.

**Pages and islands.** A handler (`routes/`) loads everything a page needs into plain structs, then renders the page's component (`ui/`) inside the layout, in one go (`ui::render`). Most of a page is static HTML. The parts that react are islands (`islands/`): the server renders them with the page and writes their props into it; in the browser, the WebAssembly (served at `/pkg/<version>/`, cached for good) brings each one to life. Islands get their texts as props (the translations stay on the server), share state through signals (`islands/shared.rs`: the cart, the open menu...), and change things on the server through *server functions* (`/api/...`, e.g. putting a variant in the cart), which only the site's own pages may call (their `Origin` is checked). Every island is also plain HTML: the add-to-cart button is a form, the cart's quantities are forms, the quote form's lists are the browser's own, so nothing depends on the WebAssembly having loaded. The theme and the cookie choice are cookies, so the server renders the page dark, or without the cookie banner, itself.

**Pictures.** Uploaded photos are stored as `storage/uploads/<key>` with a row in `uploads`, which the product, variant, promo or user points at (`image_id`, `avatar_id`); they are served at `/uploads/<key>/<file name>`, cached for good. Admin photos lose their metadata (GPS position, camera) on upload; profile pictures are re-encoded to 256×256 WebP.

## How the code is organised

Small files, one job each: a page, a table, a piece of the layout. A file's first lines (`//!`) say what it is for. Folders group what belongs together; their `mod.rs` lists the files and says how they fit.

```
src/
  main.rs            starts the server, or runs a command (reads HOST, PORT, APP_ENV, .env...)
  commands/          setup, migrate, admin, backup
  lib.rs             app(): the whole site as one service; hydrate(): the browser's entry point
  config.rs          settings: where files, database, uploads and the WebAssembly are
  state.rs           AppState, shared by all requests
  request.rs         takes /at off the URL before routing; Req = language, paths, site address, cookies
  locale.rs          Czech/German and their URLs
  i18n.rs            the translations (sapwood-i18n): locales/cs.yml, de.yml, German falls back to Czech
  assets.rs          the static files (sapwood-assets): images and CSS
  wasm.rs            the islands' WebAssembly, served at /pkg/<version>/
  session.rs         what the session cookie holds (sapwood-session): cart, checkout, last order
  forms.rs           submitted forms (sapwood-forms): token-checked, upload size limits
  shop/              cart and cart lines, putting a variant in the cart, checkout details,
                     line descriptions, VAT
    delivery/        the delivery quote and the openrouteservice distance lookup
  admin/             what the admin edits: one file per table (columns, checks),
                     record.rs (a row as form values and back), messages.rs, images.rs
                     (checking and cleaning uploaded photos), ordering.rs, dashboard.rs
  czech_time.rs      Czech time (UTC+1, summer UTC+2) for showing dates and counting days
  db/                the connection; migrations.rs (sqlx's migrator, the startup check),
                     setup.rs (a new database with the starting content), backup.rs
  storage.rs         uploaded files: the file, its `uploads` row, the record pointing at it
  accounts/          the signed-in user, passwords (bcrypt), registration checks, rate limits
    avatar/          profile pictures: cleaning uploads, keeping the EXIF data inside the WebP
  mail/              e-mails: SMTP or outbox, settings; each mail's data and plain text
  money/             Kč and € formatting; the live exchange rate
  models/            one file per table
    catalog/         product, variant, what lists need from the variants, firewood grid
    content/         promo, service, category tile, FAQ slide, review
    user.rs          accounts and their profile; login_session.rs: the sessions table
    order.rs, inquiry.rs, pricelist.rs, inquiry_options.rs, upload.rs
  ui/                the HTML: Leptos components, rendered on the server
    context.rs       what components read: t("key"), path("/kontakt"), asset("logo.webp"), the form token
    render.rs        a page inside the layout, as a response
    layout/          <head>, header, footer, mobile menu, breadcrumbs
    home/, eshop/, checkout/, account/, contact/, ...   one module per area, like routes/
    admin/           the admin's layout and pages; form.rs: its form fields
    mail/            the e-mails' HTML
    shared/          pieces used on several pages (product card, carousel, how to order...)
  islands/           the interactive parts, also compiled to WebAssembly
    cart.rs          header widget, add-to-cart, the "added" window, the cart page;
                     the cart's server functions
    page.rs          theme switch, mobile menu, sticky logo, cookie banner
    widgets.rs       accordions, carousels, Google reviews, product filters, fitting headline
    quote.rs         the quote form's rows and lists
    checkout.rs, account.rs, admin.rs   the terms box, the pickers, "really delete?"
    shared.rs        state the islands share; browser.rs: small helpers for the browser's page
  icons.rs           SVG pictures, for the pages and the islands
  views/             what pages are built from besides components: title and description
                     (page.rs), breadcrumbs, search engine links (seo.rs), the cart as
                     shown, redirects, the static error pages, text helpers
  routes/            one file per page; mod.rs is the list of all URLs
    home.rs, sluzby.rs, sortiment.rs, legal.rs, product_range.rs
    kontakt/         the page, its price list, its quote form
    cart/            cart page, add, change quantity/remove (the plain forms)
    checkout/        one file per step, the guard between steps, the order draft
    inquiries.rs     the quote request form
    auth/            sign-in, sign-out, registration
    account/         the account page, profile form, profile picture, ordering again
    admin/           the admin: guard.rs lets only admins in, params.rs reads record ids,
                     one file per section
    eshop/           listing, product page, one file per product layout
                     (firewood, lumber, simple_variant), recommendations
    components/      data used by several pages (category tiles, business data for
                     search engines, text formatting)
    server_fns.rs    /api/...: the islands' server functions, with the Origin check
    error.rs         what a failing request shows (views/error_pages.rs: public/500.html)
    rejections.rs    a refused form gets public/422.html
    headers.rs       the protective headers on every response
    public_files.rs  /icon.png and the other files of public/
    not_found.rs, system.rs, sitemap.rs, uploads.rs
sapwood/             the framework, one crate per folder (see below)
locales/             cs.yml, de.yml: every text of the site
assets/              images/ and stylesheets/ (served with fingerprints)
public/              icons and the static error pages
db/                  migrations/ (the tables, built into the program), seeds.sql and
                     seed_images.csv (the starting content for `setup`)
deploy/              release package script, systemd units, Caddyfile, backups, LAUNCH.md
tests/               requests against the whole app, like a browser would make them;
                     fixtures/data.sql is the test data, loaded into a fresh in-memory database;
                     fixtures/accounts.sql adds two accounts (password "heslo12345"), one an admin;
                     fixtures/pkg is a stand-in for the WebAssembly build
storage/             (not in git) the development database and uploads/

```

### Components

Pages are Leptos components: Rust functions marked `#[component]` that return `view! { ... }`, HTML with Rust values in braces.

| What | How |
|------|-----|
| a translation | `{t("nav.domu")}`, with a value: `{t_with("checkout.delivery_line", &[("km", "100")])}`; `t_html` for keys ending in `_html` (trusted HTML: `inner_html=t_html("...")`) |
| a link in the page's language | `href=path("/kontakt#cenik")` |
| a static file | `src=asset("logo-wm.webp")` |
| a list | `{items.into_iter().map(\|item\| view! { <li>{item.name}</li> }).collect_view()}` |
| something optional | `{maybe.map(\|value\| view! { <p>{value}</p> })}`, or `(condition).then(...)` |
| another component | `<ProductCard product=product/>` |
| a form | `<CsrfField/>` in every POST form |

Text is HTML-escaped automatically. Leptos writes `class` after the other attributes, and puts `<!>` between two text nodes that sit side by side.

An island is a component marked `#[island]`. Its props must be serializable (they travel inside the page), and it can't call `t()` or `path()`: it runs in the browser too, where the translations aren't, so the server passes it its texts and links (see the `...Texts` structs in `islands/`). Code that may only run on the server goes in `#[cfg(feature = "ssr")]`, and in server functions (`#[server]`).

### Adding a page

1. Write the component in `src/ui/` (a struct for its data, a `#[component]` for its HTML).
2. Add a handler (a function that answers the request) in `src/routes/`. It loads the data, builds a `Page` with the title and description and calls `ui::render::page(&state, &req, page, move || view! { <MyPage data/> })`.
3. Add the URL in `src/routes/mod.rs`. It works under `/at` automatically.
4. Add a test in `tests/site.rs`.

## Rust concepts you'll meet in this code

- **Ownership and borrowing** (`&str` vs `String`, `&AppState`): who owns a value and who only looks at it. The compiler checks this, so there are no use-after-free bugs and no data races.
- **`Option` and `Result`** instead of `nil` and exceptions. `?` means "return the error to my caller". See `Translations::load_dir` in sapwood-i18n.
- **Enums with `match`**: `Locale` in `locale.rs`. The compiler makes sure every language is handled.
- **Traits**: `FromRequestParts` for `Req` in `request.rs` lets a handler simply ask for `req: Req`, and axum builds it from the request.
- **`async`/`await`**: handlers are async so one small process can serve many visitors at once (tokio is the runtime).
- **`Arc`**: shared ownership across threads. `AppState` wraps everything in one `Arc`, so cloning it per request is just a counter bump.
- **`sqlx::FromRow`**: a struct per table row (`models/catalog/product.rs`); queries return typed values instead of hashes.
- **Lifetimes** (`FirewoodGrid<'a>`): the grid borrows the variants instead of copying them, and the compiler checks they live long enough. `routes/eshop/firewood.rs` has a small one: a closure returning an iterator needs `&'static str`.
- **Modules and privacy**: `mod` declares a file, `pub` makes things visible outside it, `pub use` re-exports. A child module may add methods to its parent's type (sapwood-assets' `serve.rs` adds `impl Assets`) and see its private fields.
- **Shared, changeable state** (sapwood-session's `session.rs`): `Arc<Mutex<…>>` lets the session layer and the handler see the same data; `read` and `update` keep the lock short.
- **Generics** (`Session<T>` in sapwood-session): the framework stores any session type the app defines, as long as it can be serialized; `PhantomData` remembers `T` in types that don't hold one.
- **Tower layers** (sapwood-session's `layer.rs`): the session middleware is a `Layer` that wraps every route in a `Service`; it loads the cookie before the handler and writes it after.
- **Extractors** (sapwood-forms' `extract.rs`): `impl FromRequest for FormData` means a handler that takes `form: FormData` can only run after the token check passed.
- **`Send` futures**: axum runs handlers on any thread, so nothing that isn't `Send` may live across an `.await`. `shop/delivery/route_service.rs` shows the fix for one such error (finish the URL builder into a `String` first); `#[axum::debug_handler]` explains these errors best.
- **`spawn_blocking`** (`routes/account/avatar.rs`): resizing a picture is CPU work that would stall the async threads, so it runs on a thread meant for blocking work.
- **`Box`** (`require_login` in `routes/account/mod.rs`): puts a big value on the heap so the `Result` that carries it stays small; clippy points these out.
- **Middleware** (`routes/admin/guard.rs`): a function that runs before every admin route; it answers itself (a redirect) or passes the request on with `next.run(...)`. Checking access in one place means no page can forget it.
- **Custom extractors with their own rejection** (`routes/admin/params.rs`): `RecordId` reads `{id}` from the URL and answers "not found" by itself when it isn't a number.
- **`const` tables** (`admin/products.rs`): each admin table's columns are a constant, checked by the compiler and by a test that every column is a field of its form.
- **Byte-level parsing** (`admin/images.rs`): walking a JPEG's or PNG's blocks with slices and `get(..)?`, which stops at a broken file instead of panicking.
- **Graceful shutdown** (`main.rs`): `tokio::select!` waits for whichever comes first, Ctrl+C or systemd's SIGTERM; the server then finishes the requests it is answering.
- **`OnceLock`** (`views/error_pages.rs`): a value set once at startup and read from anywhere afterwards, safely across threads.
- **Components and `view!`** (`ui/`): a component is a function; `view!` checks the HTML at compile time, and a page's whole structure becomes one (big) type. That's why `lib.rs` raises `recursion_limit`, and why a page that may show one of two things returns `.into_any()`.
- **Features and `cfg`** (`Cargo.toml`, `lib.rs`): the crate is built twice, as the server (`ssr`) and as WebAssembly for the browser (`hydrate`); `#[cfg(feature = "ssr")]` keeps the database, the files and the mail out of the browser build.
- **Signals** (`islands/`): `RwSignal` holds a value; whatever reads it in a `view!` (a `move ||` closure) updates by itself when it changes. `islands/shared.rs` gives islands on one page the same signal.
- **Server functions** (`islands/cart.rs`): `#[server]` turns an `async fn` into an HTTP endpoint on the server and a function that calls it in the browser; arguments and answers are serialized with serde.
- **Tests next to the code**: `#[cfg(test)] mod tests` at the bottom of a file; whole-app tests in `tests/`.

Good next reads: *The Rust Programming Language* (https://doc.rust-lang.org/book/), chapters 4 (ownership), 6 (enums), 9 (errors) and 10 (traits), then the axum examples (https://github.com/tokio-rs/axum/tree/main/examples).

## Sapwood

The parts of the site that any server-rendered Rust site needs are a small framework, Sapwood, in `sapwood/` (MIT OR Apache-2.0). Each crate is used by the site and tested on its own; none of them cares how the HTML is made:

| Crate | What it does |
|-------|--------------|
| `sapwood-i18n` | translations: YAML per language, dotted keys, `{name}` placeholders, fallback language (and, for MiniJinja sites, a `t()`) |
| `sapwood-assets` | fingerprinted static files, an Axum router for `/assets` (and an import map for sites with JavaScript) |
| `sapwood-session` | encrypted cookie sessions with your own data type, CSRF token, flash messages, login cookie |
| `sapwood-forms` | CSRF-checked form extractors (plain, typed with serde, uploads) and validation errors |
| `sapwood` | all of the above in one dependency |

`cargo test --workspace` tests the site and the framework together.

## History

The site started as a Ruby on Rails app (branch `main`) and was rewritten in Rust, first page for page, then made independent of it: its own database design, its own forms and admin, and the framework above. Then the HTML moved from templates to Leptos components, and the JavaScript (Turbo, Stimulus and the small scripts) to islands. The Rust version is the one that goes live.
