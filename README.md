# Pila Derflice

The website and e-shop of the Derflice sawmill, in Czech and (under `/at`)
Austrian German. It is written in Rust: [Axum](https://github.com/tokio-rs/axum)
serves the pages, [Leptos](https://leptos.dev) renders them on the server, and
the interactive parts (menus, cart, address search, photo shrinking, …) are
Leptos *islands* that run in the browser as WebAssembly. Everything else is
plain HTML and CSS, so pages work and load fast without waiting for the WASM.

## Running it locally

```sh
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --version 0.3.12 --locked
cargo install sqlx-cli --no-default-features --features sqlite   # only to change queries

cargo leptos watch                  # http://127.0.0.1:3000, rebuilds on save
cargo run -p server -- admin:create_user   # with ADMIN_EMAIL and ADMIN_PASSWORD set
```

The first start creates `storage/development.db`, runs the migrations and
loads the starting content (`seeds/seed.sql`: catalogue, price list, promos).
`cargo run -p server -- db:seed` loads it again (it refuses in production
when there is content already, unless `FORCE_SEED=1`).

Checks, the same as CI runs:

```sh
cargo fmt --all --check
cargo clippy -p server --all-targets -- -D warnings
cargo clippy -p front --target wasm32-unknown-unknown -- -D warnings
cargo test -p domain -p site --features site/ssr
```

## How the code is laid out

| Folder | What it holds |
| --- | --- |
| `crates/domain` | The shop's rules as plain Rust with no web code: money (`Czk`, euro conversion), VAT, discounts, delivery prices, the cart, the checkout steps. Most tests live here. |
| `crates/site` | The pages (`pages/`), the islands, form handlers (`forms/`), the admin (`admin/`), orders and e-mails. Built twice: with the `ssr` feature for the server, with `hydrate` for the browser. |
| `crates/server` | The binary: configuration, the router and its layers, migrations and the `admin:create_user` / `db:seed` tasks. |
| `crates/front` | The WebAssembly entry point; it only wakes the islands. |
| `migrations/` | The database schema (SQLite, run at start). |
| `seeds/` | The starting content. |
| `locales/` | The texts, `cs.yml` and `de.yml`. Each key becomes a function (`t::eshop::title(locale)`) at build time, so a missing or misspelt key is a compile error. |
| `style/` | The CSS, bundled into one file named after its content (`/assets/site-<hash>.css`, cached for a year). |
| `assets/` | The pictures and icons, served as they are. |

Database queries are checked against the schema at compile time (`sqlx::query!`).
The checked queries are saved in `.sqlx/`, so building needs no database; after
changing a query or a migration run
`DATABASE_URL=sqlite://storage/development.db cargo sqlx prepare --workspace`
and commit `.sqlx/`.

The pieces that are useful beyond this site live as separate crates in
[hospodskej/rust-crates](https://github.com/hospodskej/rust-crates):
`cuticle` (security headers), `codon` (typed translations), `refractory`
(rate limits), `thymus` (passwords and reset links), `spliceosome` (cleaning
uploaded images), `attenuator` (shrinking photos in the browser) and
`chemotaxis` (road distances from openrouteservice).

### Photos

- The site's own pictures in `assets/img` are at most twice the size they are shown at, so they stay sharp on retina screens. When adding one, shrink it first (at most ~1500 px wide for a photo, WebP).
- Product and promo photos uploaded in the admin are shrunk by the browser before upload (at most 1600 px, WebP; the `attenuator` crate), so the server needs no image library. They are stored in `storage/uploads` under a new name each time and served with "cache for a year".
- Pictures further down a page load only when the visitor scrolls near them; the main picture on the first screen loads eagerly with high priority.

### Delivery price

Delivery to an address costs a price per km (incl. VAT; 40 Kč to start with), counted there and back, one trip for the whole order. The admin sets the price in Admin > Doprava; it drives the checkout formula (`domain::delivery`) and every text that quotes it: the note on the e-shop and product pages, the checkout and the terms. Orders already placed keep the price they were placed with. The road distance from the sawmill (Derflice 66) to the address is looked up with [openrouteservice](https://openrouteservice.org) and cached in the database for 30 days per address.

Before checkout, "Kam vozíme?" asks for a town or PSČ and shows the approximate delivery price straight away. The distance (not the price, so a new price per km applies at once) stays in the session for the visit; the cart adds it to the total and the details step pre-fills the town and PSČ. Checkout recalculates with the full address, and that price is the one on the order.

- Set the `ORS_API_KEY` environment variable (locally in the git-ignored `.env`; never commit the key). Without it, or if the lookup fails, the checkout falls back to "delivery price calculated individually" and the total contains the goods only.
- The server must be allowed to reach `api.openrouteservice.org` over HTTPS.
- The free plan allows 1,000 geocoding and 2,000 route requests a day.

### Languages and URLs

The German (Austrian) site lives under `/at` (`/at/kontakt`, `/at/eshop/tramy`, ...), Czech has no prefix. The language comes from the URL only, so a shared link opens in the language it was shared in. Old `?locale=de` links redirect (301) to the `/at` URL. The admin area is Czech only.

### E-mails

Orders and inquiries are sent over SMTP (`SMTP_ADDRESS`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, from `MAIL_FROM_ADDRESS`); the shop is told at `ORDER_NOTIFICATION_EMAIL` (inquiries at `INQUIRY_NOTIFICATION_EMAIL` if set). Without `SMTP_ADDRESS` (locally) the e-mails are written to the log instead.

## Deployment

The site runs on its own server in a Docker container, deployed with
[Kamal](https://kamal-deploy.org) from GitHub Actions (Actions → Deploy).
Step-by-step setup, the DNS switch and backups: [LAUNCH.md](LAUNCH.md).

## Security

- **Repeated attempts** are limited per address (`crates/site/src/limits.rs`): login 10 per 3 minutes (and 20 per hour against one account from any address), registration 5 per hour, discount codes 10 and delivery lookups 20 per 10 minutes, inquiries, orders and password-reset requests 5 per 10 minutes.
- **Access control:** the admin needs an admin login (a middleware in front of every `/admin` route); customers only ever reach their own orders and profile picture; the cart and the last order live in the session. After signing in, only a path on this site is followed (`?return_to`).
- **Authentication:** Argon2id passwords (8+ characters); login takes as long for unknown e-mails as for known ones; the session gets a new ID at sign-in (cart kept) and is emptied at sign-out; admin logins expire after 30 days, customers stay signed in.
- **Forgotten password** (`/zapomenute-heslo`): the same answer whether or not the e-mail has an account; the e-mailed link works for 15 minutes and only until the password changes, its token leaves the address bar at once, and a new password signs the account out everywhere. Links in e-mails use `APP_HOST`, so set it in production.
- **Headers:** a strict Content-Security-Policy (inline scripts carry a per-request nonce; only Google Analytics and the footer map are allowed from outside), Permissions-Policy, `X-Frame-Options`, HSTS in production, `Cache-Control: no-store` on the admin, account and checkout, `noindex` on the admin.
- **Uploads:** profile pictures are checked and cleaned on the server (2 MB, 25 megapixels, JPG/PNG/WebP only; metadata and anything appended to the image removed). Requests are capped in size: 3 MB for the profile picture, 16 MB for admin uploads, 2 MB for everything else.
- **Injection:** every query is a fixed, compile-time-checked SQL statement with bound values; texts from the database pass an allow-list of formatting tags; islands build their content as DOM nodes, never as HTML strings.
- **Checks:** `cargo audit` before a release. Dependabot (`.github/dependabot.yml`) opens weekly pull requests for crate updates; turn on "Dependabot security updates" in the repository's settings for immediate security fixes.

## To-dos

The open to-dos live in the code, as `TODO` comments where the work will happen (e.g. the unfinished FAQ answers in `locales/cs.yml`). List them all with:

```
git grep -n "TODO"
```
