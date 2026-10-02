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
