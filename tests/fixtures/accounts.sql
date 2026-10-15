-- Accounts. The password of both is "heslo12345" (hash made by Rails' bcrypt).
INSERT INTO users (id, email_address, password_digest, role, first_name, last_name, phone, billing_street, billing_city,
  billing_zip, billing_country, company_purchase, delivery_address_different, created_at, updated_at) VALUES
  (1, 'zakaznik@example.com', '$2a$04$e3UTqAmP.3xrk/k1K/dT8OnaE.WX2BB7y37fzLvGlm.kH4Ncu3PDy', 'customer', 'Karel', 'Dvořák',
   '+43 660 1234', 'Hauptplatz 1', 'Retz', '2070', 'at', 0, 0, '2025-03-01 09:00:00', '2025-03-01 09:00:00'),
  (2, 'admin@piladerflice.cz', '$2a$04$e3UTqAmP.3xrk/k1K/dT8OnaE.WX2BB7y37fzLvGlm.kH4Ncu3PDy', 'admin', NULL, NULL,
   NULL, NULL, NULL, NULL, NULL, 0, 0, '2025-01-01 09:00:00', '2025-01-01 09:00:00');

-- A past order of the customer: one variant still sold, one gone.
INSERT INTO orders (id, user_id, first_name, last_name, email, phone, billing_street, billing_city, billing_zip,
  billing_country, shipping_method, payment_method, items_snapshot, subtotal_czk, vat_czk, total_czk, locale,
  created_at, updated_at) VALUES
  (500, 1, 'Karel', 'Dvořák', 'zakaznik@example.com', '+43 660 1234', 'Hauptplatz 1', 'Retz', '2070', 'at', 'pickup', 'cash',
   '[{"catalog_variant_id":"10","product_key":"smrk","image":null,"unit_price_czk":9000,"specs":[],"quantity":2},{"catalog_variant_id":"999","product_key":"zmizely","image":null,"unit_price_czk":100,"specs":[],"quantity":1}]',
   18100, 3141, 18100, 'cs', '2026-05-01 10:00:00', '2026-05-01 10:00:00'),
  (501, 2, 'Admin', '', 'admin@piladerflice.cz', '1', 'A', 'B', '1', 'cz', 'pickup', 'cash',
   '[{"catalog_variant_id":"10","product_key":"smrk","image":null,"unit_price_czk":9000,"specs":[],"quantity":1}]',
   9000, 1562, 9000, 'cs', '2026-05-02 10:00:00', '2026-05-02 10:00:00');
