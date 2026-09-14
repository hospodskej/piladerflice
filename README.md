# README for Pila Derflice

- Install Ruby on Rails
- `bundle install`
- `rails db:migrate`
- `rails db:seed`
- `rails server`

---

Patch 0.8.26: Fixed Akční nabídka's shadow overflowing past the section's right edge on desktop (reserved room for its peek-out instead)  
Patch 0.8.25: Fixed Akční nabídka image/shadow being fixed-width and overflowing their column, squeezing the text next to it  
Patch 0.8.24: Aligned all homepage sections to a single 1200px content width, removing redundant per-section max-widths/padding  
Patch 0.8.23: FAQ CTA button now vertically centered; equal-height FAQ boxes so a 2-line question doesn't dwarf its neighbor  
Patch 0.8.22: Added homepage FAQ section ("Vámi nejčastěji kladené dotazy") with 3 categories and a contact CTA  
Patch 0.8.21: "Co je nového?" gallery block now wider than the text column (0.8.20 only resized within a fixed-width box)  
Patch 0.8.20: "Co je nového?" main image made wider than the two stacked images  
Patch 0.8.19: Added "Co je nového?" section (image gallery + text) between the delivery map and "Jak si u nás objednat?"  
Patch 0.8.18: Reverted carousel hover borders (0.8.17); removed "Co vám můžeme nabídnout?" image border/hover (0.8.6), kept images clickable  
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
