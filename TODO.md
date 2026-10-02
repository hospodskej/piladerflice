# Roadmap

Bigger things we've talked about but haven't built yet. Not a changelog (see README.md for that) - this is where ideas go before they become patches.

---

## Internal team app

A separate tool for the team: communication, a task list, and a quick-order system connected to live orders/catalog/stock - not a duplicate of the storefront's data.

- Team communication + task list
- Tablet-friendly quick order entry using product numbers, for fast in-person/phone orders
- Needs to read/write the same orders, catalog, and stock data as the main site, live - not a synced copy
- Leaning toward building it in Rails (same stack, shares models/business logic directly) rather than a separate language, as either its own app or a subdomain of this one

## AI chatbot

A chatbot for the site, most likely answering product/order questions for customers.

- Leaning toward Python (Django) rather than Ruby, since it's a better fit for the AI/ML tooling a chatbot needs
- Would run as its own separate app, not bolted onto this Rails app
- Scope (what it can answer, whether it can place orders, etc.) still undecided

## Order status tracking

Customers can see their order history now but not whether it's been processed, dispatched, or delivered - they currently have to call to ask.

- Add a status field to orders (e.g. processing / dispatched / delivered) with an admin UI to update it
- Show the status on the account page's order history
- Natural extension of the order history/reorder work already built

## On-site customer reviews

A `Review` model already exists, currently just holding imported Google reviews. Letting actual buyers leave their own review (tied to their account/order) would build more trust than imported reviews alone.

- Needs light moderation before a review goes public
- Likely gated to customers who actually have an order for that product

## Low-stock alerts for admin

This is a physical sawmill with real inventory - right now nobody gets notified when a product variant's `in_stock` flips to false; it just has to be noticed manually.

- Notify admin (email or an admin-panel flag) when a variant goes out of stock

## Site search

The catalog is growing (species, dimensions, grades) and currently has no search, only category browsing.

- A search bar across products/variants once the catalog outgrows simple category pages

## Custom 404 page

`public/404.html` is still the generic Rails default (plain English, not styled to match the site) rather than something branded and in Czech.

## robots.txt

`public/robots.txt` is still just the default Rails placeholder comment, with no real rules in it.

## Team mascot

An illustrated mascot for the brand - could double as the face of the 404 page, loading states, etc.

## Team photo

A real photo of Štěpán Merta / the team for the "O nás" section, replacing the current stock/illustration imagery.
