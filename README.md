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

## Security

- **Repeated attempts** are limited per address (`rate_limit`): login 10 per 3 minutes (and 20 per hour against one account from any address), registration 5 per hour, discount codes 10 and delivery lookups 20 per 10 minutes, inquiries, orders and password-reset requests 5 per 10 minutes. Counters live in the Rails cache, which in production is a file store in `tmp/cache` shared by all server processes on the machine.
- **Access control:** the admin needs an admin login (`Admin::BaseController`); customers only ever reach their own orders and profile picture; the cart and the last order live in the browser session. `test/integration/idor_test.rb` tries every admin route as a guest and a customer. Active Storage's direct uploads are closed (`config/routes.rb`). After signing in, only a path on this site is followed (`?return_to`).
- **Authentication:** bcrypt passwords (8+ characters); login answers equally fast for unknown e-mails (`authenticate_by`); the browser session gets a new ID at sign-in (cart kept) and is emptied at sign-out; admin logins expire after 30 days, customers stay signed in.
- **Forgotten password** (`/zapomenute-heslo`, `PasswordsController`): the same answer whether or not the e-mail has an account; the e-mailed link (Rails' `password_reset_token`) works for 15 minutes and only until the password changes, its token leaves the address bar at once, and a new password signs the account out everywhere. Links in e-mails use `APP_HOST`, so set it in production.
- **Headers:** a strict Content-Security-Policy (`config/initializers/content_security_policy.rb`; inline scripts carry the nonce, only Google Analytics and the footer map are allowed from outside), Permissions-Policy, HSTS via `force_ssl`, `Cache-Control: no-store` on the admin, account and checkout, `noindex` on the admin. In production set `APP_HOSTS` (e.g. `piladerflice.cz,www.piladerflice.cz`) so other Host headers are refused.
- **Injection:** queries go through Active Record only; texts from the database pass an allow-list of formatting tags (`formatted_text`, `pricelist_html`); the custom dropdowns build options with `textContent`.
- **Checks:** `brakeman` (no warnings) and `bundler-audit check --update` (no known vulnerable gems) before a release. Dependabot (`.github/dependabot.yml`) opens weekly pull requests for gem updates; turn on "Dependabot security updates" in the repository's settings for immediate security fixes.

## Roadmap

Bigger things we've talked about but haven't built yet.

### Two-step login for the admin

After the password, a second step for admin accounts: a 6-digit code from an authenticator app (Google / Microsoft Authenticator) with one-time recovery codes, or a code sent by e-mail. Not decided yet which.

### Internal team app

A separate tool for the team: communication, a task list, and a quick-order system connected to live orders/catalog/stock - not a duplicate of the storefront's data.

- Team communication + task list
- Tablet-friendly quick order entry using product numbers, for fast in-person/phone orders
- Needs to read/write the same orders, catalog, and stock data as the main site, live - not a synced copy
- Leaning toward building it in Rails (same stack, shares models/business logic directly) rather than a separate language, as either its own app or a subdomain of this one

### AI chatbot

A chatbot for the site, most likely answering product/order questions for customers.

- Leaning toward Python (Django) rather than Ruby, since it's a better fit for the AI/ML tooling a chatbot needs
- Would run as its own separate app, not bolted onto this Rails app
- Scope (what it can answer, whether it can place orders, etc.) still undecided

### Order status tracking

Customers can see their order history now but not whether it's been processed, dispatched, or delivered - they currently have to call to ask.

- Add a status field to orders (e.g. processing / dispatched / delivered) with an admin UI to update it
- Show the status on the account page's order history
- Natural extension of the order history/reorder work already built

### On-site customer reviews

A `Review` model already exists, currently just holding imported Google reviews. Letting actual buyers leave their own review (tied to their account/order) would build more trust than imported reviews alone.

- Needs light moderation before a review goes public
- Likely gated to customers who actually have an order for that product

### Google Search Console

Not set up yet - nobody has done it since the site isn't live. The old verification tag was removed from the layout, so this starts from scratch once the site is live.

- Add the live domain at https://search.google.com/search-console and verify ownership (the DNS TXT record option needs no code change; the HTML-tag option means adding Google's new `google-site-verification` meta tag to `app/views/layouts/application.html.erb`)
- Under Sitemaps, submit `sitemap.xml` (the site generates it automatically; `robots.txt` already points to it)
- Later, check the Pages report for pages Google couldn't index

### Google Analytics

The tracking code is already in the layout (measurement ID `G-CMJ46E7F3G`, loaded with Consent Mode so nothing is stored until a visitor clicks "Přijmout vše"), and a `purchase` event fires on the order confirmation page. It has never been checked against live traffic, so once the site is live:

- Confirm the measurement ID belongs to the right Google Analytics property and the data stream points at the live domain
- Accept cookies on the live site and check the Realtime report shows the visit
- Place a test order and check the `purchase` event arrives (mark it as a key event/conversion if wanted)
- Set the data retention in Admin > Data collection and modification > Data retention to match what the privacy policy says (currently up to 14 months)
- Link the property to Google Search Console once that is set up

### SEO

The basics are in place (robots.txt, sitemap.xml, and `/llms.txt`: a Markdown overview for AI assistants with the business facts, every active product with its "from" price and the delivery price per km, generated from the catalogue), but the on-page SEO work is still open. Meta descriptions and unique page titles are done (`page_meta` helper, texts under `meta:` in the locale files). Local business schema (JSON-LD on the home and contact pages) and canonical/`hreflang` links are done too. Still open: the social share image entry below (a designer is making the image). Once that is done, also:

- Check heading structure (one `h1` per page) and image `alt` texts on the main pages
- Run Lighthouse/PageSpeed on the live site and fix whatever it flags

### Update privacy policy after launch

The privacy policy (`legal.privacy_content_html` in `config/locales/cs.yml` and `de.yml`) was drafted before the server was live. Once the server is up and running, update it with the real hosting and e-mail providers, confirm the retention periods and the Google Analytics retention setting, and have it reviewed (ideally by a lawyer).

### Use the mascots elsewhere

The mascot illustrations live in `app/assets/images/mascot/`. The shrugging one (face filled light blue, so it looks the same in light and dark mode) is on the 404 page, in front of a black and white photo. The thumbs-up one is on the order confirmation page; the waving and hands-on-hips ones aren't used yet. They could show up on the empty cart, loading states and the "O nás" section.

### Team photo

A real photo of Štěpán Merta / the team for the "O nás" section, replacing the current stock/illustration imagery.

### Social share image

The Open Graph / Twitter tags are in place (title, description, address and language of each page) but there are no cards yet, so a shared link has no picture. A designer is making them: two files, **1200x630 px**, WebP (PNG or JPG also work), ideally under 300 KB, one per language:

- `app/assets/images/share/share-cs.webp` - used for the Czech site
- `app/assets/images/share/share-de.webp` - used for the Austrian German site (`/at`)

Drop them in and restart; nothing else to change. A language without a card just has no picture.
