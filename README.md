# README for Pila Derflice

- Install Ruby on Rails
- `bundle install`
- `rails db:migrate`
- `rails db:seed`
- `rails server`

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

### Site search

The catalog is growing (species, dimensions, grades) and currently has no search, only category browsing.

- A search bar across products/variants once the catalog outgrows simple category pages

### Custom 404 page

`public/404.html` is still the generic Rails default (plain English, not styled to match the site) rather than something branded and in Czech.

### robots.txt

`public/robots.txt` is still just the default Rails placeholder comment, with no real rules in it.

### Team mascot

An illustrated mascot for the brand - could double as the face of the 404 page, loading states, etc.

### Team photo

A real photo of Štěpán Merta / the team for the "O nás" section, replacing the current stock/illustration imagery.

### Meta descriptions

No page sets one, so Google just auto-grabs random text from the page for the search result snippet instead of something written to get clicks.

### Unique page titles

Only one view sets a custom `<title>` - every other page (eshop, every product page, cart, checkout, account, etc.) shows the generic "Pila Derflice" title.

### Clickable phone/email

The phone number and email in the header and footer are plain text, not `tel:`/`mailto:` links (kontakt page has a working mailto, but it's the only one).

### Social share image

No `og:image`/`og:title` meta tags, so a link to the site shared in WhatsApp/Facebook/iMessage shows no useful preview card.

### Privacy policy page

There's a terms page and a cookies policy, but no separate privacy policy - relevant now that the site collects accounts and checkout data.

### Local business schema

No JSON-LD structured data (LocalBusiness schema), which helps local SEO/Google Maps visibility.

---

Patch 0.8.80: Real dark mode palette (was a color-invert filter), fixed 500 on terms/cookie-policy pages  
Patch 0.8.79: Made bestseller, TOP kategorie, and recommendation images clickable  
Patch 0.8.77: Admin-uploaded images get losslessly shrunk on upload (gems only, no server dependency)  
Patch 0.8.76: Converted every static site image to WebP ahead of time (168MB -> 29MB), no server dependency needed  
Patch 0.8.74: SVG flags for phone dropdown  
Patch 0.8.73: Phone prefix dropdown  
Patch 0.8.72: Edit profile page, removed old admin login  
Patch 0.8.71: Account dashboard redesign  
Patch 0.8.70: Order history & reorder  
Patch 0.8.69: Fixed registration error messages  
Patch 0.8.68: Admin "back to site" link  
Patch 0.8.67: Customer login & registration  
Patch 0.8.66: Gold review stars in dark mode  
Patch 0.8.65: Sticky header logo  
Patch 0.8.64: Softer dark mode colors  
Patch 0.8.63: Renamed admin promo section  
Patch 0.8.62: Renamed hero badge  
Patch 0.8.61: Live bestseller data on homepage  
Patch 0.8.60: Fixed image upload crash  
Patch 0.8.59: Per-variant product images  
Patch 0.8.58: Real image uploads  
Patch 0.8.57: Weekly rotating promos  
Patch 0.8.56: Google Search Console setup  
Patch 0.8.55: GA4 purchase tracking  
Patch 0.8.54: Header & analytics tweaks  
Patch 0.8.53: Added dark mode  
Patch 0.8.52: Cookie consent banner  
Patch 0.8.51: Polished eshop cards  
Patch 0.8.50: Reverted header size change  
Patch 0.8.49: Removed firewood qty stepper  
Patch 0.8.48: Hero title wrap fix  
Patch 0.8.47: Sleeker promo overlay  
Patch 0.8.46: Fixed FAQ accordion bug  
Patch 0.8.45: Fixed promo card crop  
Patch 0.8.44: Lumber dimension filter  
Patch 0.8.43: Fixed eshop card overflow  
Patch 0.8.42: Redesigned lumber cards  
Patch 0.8.41: Checkout terms error message  
Patch 0.8.40: Added terms & conditions  
Patch 0.8.39: Mobile overhaul  
Patch 0.8.38: Centered reviews row  
Patch 0.8.37: Hero button alignment fix  
Patch 0.8.36: Real reviewer photos  
Patch 0.8.35: Reviews carousel fix  
Patch 0.8.34: Fixed carousel overlap bug  
Patch 0.8.33: Resized review cards  
Patch 0.8.32: Fixed reviews carousel loop  
Patch 0.8.31: Added Google reviews section  
Patch 0.8.30: Centered promo text  
Patch 0.8.29: Added truck fleet info  
Patch 0.8.28: Extended homepage alignment  
Patch 0.8.27: Carousel fleet info + dots  
Patch 0.8.26: Fixed promo shadow overflow  
Patch 0.8.25: Fixed promo image overflow  
Patch 0.8.24: Unified homepage width  
Patch 0.8.23: FAQ box alignment fix  
Patch 0.8.22: Added homepage FAQ section  
Patch 0.8.21: Widened gallery block  
Patch 0.8.20: Widened "Co je nového" image  
Patch 0.8.19: Added "Co je nového" section  
Patch 0.8.18: Reverted carousel borders  
Patch 0.8.17: Blue carousel hover borders  
Patch 0.8.16: Lumber dims cm to mm  
Patch 0.8.15: Accordion hover color fix  
Patch 0.8.14: Numeric-only custom values  
Patch 0.8.13: Custom values keep units  
Patch 0.8.12: Custom values merge to dropdown  
Patch 0.8.11: Limited custom value field  
Patch 0.8.10: Fixed admin column misalignment  
Patch 0.8.9: Dashboard stats + order delete  
Patch 0.8.8: Admin-editable kalkulace options  
Patch 0.8.7: Added test suite, 2 bugfixes  
Patch 0.8.6: Clickable "Co vám můžeme nabídnout" images  
Patch 0.8.5: Clickable eshop product images  
Patch 0.8.4: Randomized eshop product recommendations  
Patch 0.8.3: Fixed product image cropping on eshop/sortiment  
Patch 0.8.2: Removed unused CSS, images & JS  
Patch 0.8.1: Kalkulace form now saves inquiries and shows them in admin  
Patch 0.8.0: Admin migration  
Patch 0.7.8: SMTP Ready  
Patch 0.7.7: Preparation for SMTP  
Patch 0.7.6: Added checkout page  
Patch 0.7.5: Fixed "do košíku" buttons on eshop  
Patch 0.7.4: Fixed cart button + eshop routing  
Patch 0.7.3: Cart button  
Patch 0.7.2: Cart UI fixes 2  
Patch 0.7.1: Cart UI fixes  
Patch 0.7.0: Added Cart and Eshop currency  
Patch 0.6.3: UI/UX fixes, clickable icon  
Patch 0.6.2: Added currency converter  
Patch 0.6.1: Added README.md updates  
Patch 0.6.0: Added translation for Austrian customers  
Patch 0.5.0: Prepared for translation  
Patch 0.4.0: Added eshop  
Patch 0.3.0: Added breadcrumbs  
Patch 0.2.4: UI/UX fixes  
Patch 0.2.3: UI/UX fixes  
Patch 0.2.2: UI/UX fixes  
Patch 0.2.1: Finished Main, Kontakty & Ceník pages  
Patch 0.2.0: Set up Main page  
Patch 0.1.0: Migrated from Static website into Ruby on Rails
