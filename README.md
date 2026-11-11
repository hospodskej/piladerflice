# README for Pila Derflice

- Install Ruby on Rails
- `bundle install`
- `rails db:migrate`
- `rails db:seed`
- `rails admin:create_user`
- `rails server`

### Requirements

- Profile pictures are checked and cleaned in the app, in plain Ruby (2 MB, 25 megapixels, JPG/PNG/WebP only; metadata and anything appended to the image removed, see `AvatarSanitizer`), but also set a request body limit on the web server / reverse proxy in front of Rails (for example `client_max_body_size 3m;` in nginx), so oversized uploads are cut off before they reach the app.

### Photos

- The site's own pictures in `app/assets/images` are at most twice the size they are shown at, so they stay sharp on retina screens. When adding one, shrink it first (at most ~1500 px wide for a photo, WebP).
- Product and promo photos uploaded in the admin are shrunk by the browser before upload (at most 1600 px, WebP; `image_shrink_controller.js`), so the server needs no image library. They are served with "cache for a year" (a new photo gets a new address).
- Pictures further down a page load only when the visitor scrolls near them. A picture on the first screen should say `loading: "eager"` (the main one also `fetchpriority: "high"`).
- A database seeded before the images were shrunk still holds the big copies: run `bin/rails images:shrink_seeded` once (it only swaps the photos that came from the seeds; uploaded ones are left alone).

### Delivery price

Delivery to an address costs a price per km (incl. VAT; 40 Kč to start with), counted there and back, one trip for the whole order. The admin sets the price in Admin > Doprava (`Setting.delivery_czk_per_km`); it drives the checkout formula (`DeliveryQuote`) and every text that quotes it: the note on the e-shop and product pages, the checkout and the terms. Orders already placed keep the price they were placed with. The road distance from the sawmill (Derflice 66) to the delivery address is looked up with [openrouteservice](https://openrouteservice.org) by `DeliveryRouteService` and cached for 30 days per address. The checkout summary shows the price as approximate and includes it in the total; the order stores it in `delivery_distance_km` (round trip) and `delivery_czk`.

Before checkout, "Kam vozíme?" (the delivery note on the e-shop, product pages and cart; `shared/_delivery_note`) asks for a town or PSČ and shows the approximate delivery price straight away. `DeliveryRouteService.estimate` finds the town (places only, in the chosen country) and routes to it; the result is cached for 30 days per place. `DeliveryEstimate` keeps the distance (not the price, so a new price per km applies at once) in the session for the visit. The cart and the cart dropdown add it to the total, the shipping step shows it, and the details step pre-fills the town and PSČ. Checkout still recalculates with the full address, and that price is the one on the order.

- Set the `ORS_API_KEY` environment variable (locally in the git-ignored `.env`; never commit the key). Without it, or if the lookup fails, the checkout falls back to "delivery price calculated individually" and the total contains the goods only.
- The server must be allowed to reach `api.openrouteservice.org` over HTTPS.
- The free plan allows 1,000 geocoding and 2,000 route requests a day, which is plenty for one order per lookup.

### Languages and URLs

The German (Austrian) site lives under `/at` (`/at/kontakt`, `/at/eshop/tramy`, ...), Czech has no prefix. Translations are German (`de` locale); `LocalizedPath::SEGMENTS` maps the locale to the URL prefix, and the alternates are announced as `de-AT`. The language comes from the URL only, so a shared link opens in the language it was shared in. Old `?locale=de` links redirect (301) to the `/at` URL. In views, link with the route helpers (language is added automatically) or, for stored or hand-written paths, with `localized_path("/kontakt#cenik")`. The admin area is Czech only.

## Deployment

The site runs on its own server in a Docker container, deployed with
[Kamal](https://kamal-deploy.org) from GitHub Actions (Actions → Deploy).
Step-by-step setup, the DNS switch and backups: [LAUNCH.md](LAUNCH.md).

## Security

- **Repeated attempts** are limited per address (`rate_limit`): login 10 per 3 minutes (and 20 per hour against one account from any address), registration 5 per hour, discount codes 10 and delivery lookups 20 per 10 minutes, inquiries, orders and password-reset requests 5 per 10 minutes. Counters live in the Rails cache, which in production is a file store in `tmp/cache` shared by all server processes on the machine.
- **Access control:** the admin needs an admin login (`Admin::BaseController`); customers only ever reach their own orders and profile picture; the cart and the last order live in the browser session. `test/integration/idor_test.rb` tries every admin route as a guest and a customer. Active Storage's direct uploads are closed (`config/routes.rb`). After signing in, only a path on this site is followed (`?return_to`).
- **Authentication:** bcrypt passwords (8+ characters); login answers equally fast for unknown e-mails (`authenticate_by`); the browser session gets a new ID at sign-in (cart kept) and is emptied at sign-out; admin logins expire after 30 days, customers stay signed in.
- **Forgotten password** (`/zapomenute-heslo`, `PasswordsController`): the same answer whether or not the e-mail has an account; the e-mailed link (Rails' `password_reset_token`) works for 15 minutes and only until the password changes, its token leaves the address bar at once, and a new password signs the account out everywhere. Links in e-mails use `APP_HOST`, so set it in production.
- **Headers:** a strict Content-Security-Policy (`config/initializers/content_security_policy.rb`; inline scripts carry the nonce, only Google Analytics and the footer map are allowed from outside), Permissions-Policy, HSTS via `force_ssl`, `Cache-Control: no-store` on the admin, account and checkout, `noindex` on the admin. In production set `APP_HOSTS` (e.g. `piladerflice.cz,www.piladerflice.cz`) so other Host headers are refused.
- **Injection:** queries go through Active Record only; texts from the database pass an allow-list of formatting tags (`formatted_text`, `pricelist_html`); the custom dropdowns build options with `textContent`.
- **Checks:** `brakeman` (no warnings) and `bundler-audit check --update` (no known vulnerable gems) before a release. Dependabot (`.github/dependabot.yml`) opens weekly pull requests for gem updates; turn on "Dependabot security updates" in the repository's settings for immediate security fixes.

## To-dos

The open to-dos live in the code, as `TODO` comments where the work will happen (e.g. order statuses in `app/models/order.rb`, the unfinished FAQ answers in `config/locales/cs.yml`). List them all with:

```
git grep -n "TODO"
```
