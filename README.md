# README for Pila Derflice

- Install Ruby on Rails
- `bundle install`
- `rails db:migrate`
- `rails db:seed`
- `rails server`

### Requirements

- The **libvips** system library, 8.15 or newer preferred (e.g. `apt install libvips` / `brew install vips`). Profile pictures are re-encoded with it (see `AvatarSanitizer`).
- Profile pictures are checked in the app (2 MB, 25 megapixels, JPG/PNG/WebP only), but also set a request body limit on the web server / reverse proxy in front of Rails (for example `client_max_body_size 3m;` in nginx), so oversized uploads are cut off before they reach the app.

### Languages and URLs

The German (Austrian) site lives under `/at` (`/at/kontakt`, `/at/eshop/tramy`, ...), Czech has no prefix. Translations are German (`de` locale); `LocalizedPath::SEGMENTS` maps the locale to the URL prefix, and the alternates are announced as `de-AT`. The language comes from the URL only, so a shared link opens in the language it was shared in. Old `?locale=de` links redirect (301) to the `/at` URL. In views, link with the route helpers (language is added automatically) or, for stored or hand-written paths, with `localized_path("/kontakt#cenik")`. The admin area is Czech only.

## Roadmap

Bigger things we've talked about but haven't built yet.

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

The basics are in place (robots.txt and sitemap.xml), but the on-page SEO work is still open. Meta descriptions and unique page titles are done (`page_meta` helper, texts under `meta:` in the locale files). Local business schema (JSON-LD on the home and contact pages) and canonical/`hreflang` links are done too. Still open: the social share image entry below (a designer is making the image). Once that is done, also:

- Check heading structure (one `h1` per page) and image `alt` texts on the main pages
- Run Lighthouse/PageSpeed on the live site and fix whatever it flags

### Update privacy policy after launch

The privacy policy (`legal.privacy_content_html` in `config/locales/cs.yml` and `de.yml`) was drafted before the server was live. Once the server is up and running, update it with the real hosting and e-mail providers, confirm the retention periods and the Google Analytics retention setting, and have it reviewed (ideally by a lawyer).

### Use the mascots elsewhere

The three mascot illustrations live in `app/assets/images/mascot/` and currently appear on the 404 page (one picked at random) and the order confirmation page (the thumbs-up one). They could also show up on the empty cart, loading states and the "O nás" section.

### Team photo

A real photo of Štěpán Merta / the team for the "O nás" section, replacing the current stock/illustration imagery.

### Social share image

The Open Graph / Twitter tags are in place (title, description, address and language of each page) but there are no cards yet, so a shared link has no picture. A designer is making them: two files, **1200x630 px**, WebP (PNG or JPG also work), ideally under 300 KB, one per language:

- `app/assets/images/share/share-cs.webp` - used for the Czech site
- `app/assets/images/share/share-de.webp` - used for the Austrian German site (`/at`)

Drop them in and restart; nothing else to change. A language without a card just has no picture.
