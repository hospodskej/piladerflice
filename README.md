# README for Pila Derflice

- Install Ruby on Rails
- `bundle install`
- `rails db:migrate`
- `rails db:seed`
- `rails server`
---

Patch 0.8.66: Google review stars are now gold instead of brown in dark mode, in both the hero and the reviews section  
Patch 0.8.65: Desktop header logo now sticks to the top, centered, once you scroll past the full header  
Patch 0.8.64: Dark mode background/header/buttons are now a softer dark gray instead of near-black, and the flag icons show true bright colors again  
Patch 0.8.63: Renamed the "Akční nabídka" admin section (sidebar, titles, messages) to match the new "Doporučujeme" naming  
Patch 0.8.62: Renamed the hero card's top-left badge from "Akční nabídka" to "Doporučujeme"  
Patch 0.8.61: "Nejprodávanější produkty" on the homepage now pulls live title/price/photo from the matching eshop product instead of separate hardcoded content  
Patch 0.8.60: Fixed a crash when saving a product/promo/variant edit with a new image and a blank dropdown field  
Patch 0.8.59: Admins can now upload a separate image per product variant (e.g. a different photo per firewood amount), falling back to the product's default image  
Patch 0.8.58: Admins can now upload real image files for products and promos instead of typing a filename  
Patch 0.8.57: Akční nabídka now rotates weekly, with a new admin page to manage the pool  
Patch 0.8.56: Added Google Search Console site verification meta tag  
Patch 0.8.55: Added GA4 purchase conversion tracking on the order confirmation page  
Patch 0.8.54: Centered header nav, moved mobile dark mode toggle into sidebar, added Google Analytics  
Patch 0.8.53: Added dark mode with a sun/moon toggle in the header  
Patch 0.8.52: Added a cookie consent banner (accept all/necessary only/decline) and cookie policy page  
Patch 0.8.51: Polished eshop product cards — matched button heights, tightened spec spacing, smaller cards  
Patch 0.8.50: Reverted the header-size unification from 0.8.48, kept the hero 2-line fix  
Patch 0.8.49: Removed the quantity stepper on palivové dřevo product cards  
Patch 0.8.48: Hero title now wraps to 2 lines, unified all main section headers to one size  
Patch 0.8.47: Sleeker Akční nabídka overlay — price moved above the button, shorter blue area  
Patch 0.8.46: Fixed FAQ accordion items stretching their row neighbor when opened  
Patch 0.8.45: Fixed Akční nabídka hero card's cropped badge and wrapping title/button  
Patch 0.8.44: Added a dimension/grade filter to stavební řezivo product pages  
Patch 0.8.43: Fixed eshop product cards overflowing at laptop widths, smaller cards and better alignment  
Patch 0.8.42: Redesigned stavební řezivo cards with grade/dimensions and a quantity stepper, admin updated to match  
Patch 0.8.41: Checkout now shows a clear Czech error when you try to submit without accepting terms  
Patch 0.8.40: Added real obchodní podmínky (CZ + DE) matching the actual checkout flow  
Patch 0.8.39: Site-wide mobile overhaul — fixed overflow bugs across eshop, sortiment, cart and checkout  
Patch 0.8.38: Google rating + features row now vertically centered against TOP Kategorie  
Patch 0.8.37: Hero section realigned so the CTA buttons' bottom matches the Akční nabídka card's bottom  
Patch 0.8.36: Wired up real reviewer photos in the seed data  
Patch 0.8.35: Reviews carousel now pre-renders the next/previous card so it slides in instead of popping in  
Patch 0.8.34: Fixed a box-sizing bug causing right-arrow overlap, shorter cards, centered header, smoother rotation  
Patch 0.8.33: Google review cards resized to clear the arrows, added optional reviewer photos, transparent reviews header  
Patch 0.8.32: Google reviews carousel now truly loops — cards rotate instead of just resetting position  
Patch 0.8.31: Added a Google reviews section above the footer, DB-backed with an infinite-loop scroll carousel  
Patch 0.8.30: Akční nabídka text now vertically centered to match the image  
Patch 0.8.29: Added truck fleet info to Doprava service; trimmed carousel vehicle descriptions for more white space  
Patch 0.8.28: Extended the homepage's 1200px alignment  
Patch 0.8.27: "Co by vás mohlo zajímat?" carousel: replaced slides with vehicle fleet info, added indicator dots  
Patch 0.8.26: Fixed Akční nabídka's shadow overflowing past the section's right edge on desktop  
Patch 0.8.25: Fixed Akční nabídka image/shadow being fixed-width and overflowing their column  
Patch 0.8.24: Aligned all homepage sections to a single 1200px content width  
Patch 0.8.23: FAQ CTA button now vertically centered; equal-height FAQ boxes  
Patch 0.8.22: Added homepage FAQ section with 3 categories and a contact CTA  
Patch 0.8.21: "Co je nového?" gallery block now wider than the text column  
Patch 0.8.20: "Co je nového?" main image made wider than the two stacked images  
Patch 0.8.19: Added "Co je nového?" section between the delivery map and "Jak si u nás objednat?"  
Patch 0.8.18: Reverted carousel hover borders; removed some stuff  
Patch 0.8.17: All carousels site-wide now get a brand blue border on hover  
Patch 0.8.16: Stavební řezivo's Výška/Šířka options converted from cm to mm, numbers scaled ×10  
Patch 0.8.15: "Jaké služby poskytujeme?" accordion items turn brand blue (border + text) on hover  
Patch 0.8.14: Custom values on unit-based kalkulace fields now only accept numbers, preventing duplicate units  
Patch 0.8.13: Custom kalkulace values now keep the field's unit (e.g. "111 cm"), matching the presets  
Patch 0.8.12: Kalkulace custom values now merge back into the dropdown on blur/Enter, still editable  
Patch 0.8.11: Palivové dřevo kalkulace form now only offers a custom value on Množství  
Patch 0.8.10: Fixed misaligned Hodnota (DE) column on the kalkulace options admin page  
Patch 0.8.9: Dashboard order stats by month, delete objednávky from admin  
Patch 0.8.8: Admin-editable kalkulace form options, new stavební řezivo form, custom/vlastní values  
Patch 0.8.7: Added a real test suite; fixed 2 bugs it caught (cheapest-price sorting, test-env mail/session config)  
Patch 0.8.6: Homepage "Co vám můžeme nabídnout?" images now clickable, blue hover border  
Patch 0.8.5: Removed old code comments & made eshop product images clickable  
Patch 0.8.4: Randomized eshop product recommendations  
Patch 0.8.3: Fixed product image cropping on eshop/sortiment  
Patch 0.8.2: Removed unused CSS, images & JS  
Patch 0.8.1: UI fixes  
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
