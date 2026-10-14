-- Test data, loaded into a fresh in-memory database for each test.
-- Like the Rails fixtures: a little of everything the pages show.

INSERT INTO catalog_products (id, key, title, title_de, type_label, type_label_de, subtitle, description, description_de,
  drying_note, image_alt, template, category, hardness, position, active, created_at, updated_at) VALUES
  (1, 'smrk', 'Smrk', 'Fichte', 'Palivové dřevo', 'Brennholz', 'Měkké dřevo', 'Suché <b>smrkové</b> dřevo.', 'Trockenes Fichtenholz.',
   'Sušeno 2 roky', 'Smrkové polena', 'firewood', 'palivove', 'soft', 0, 1, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (2, 'dub', 'Dub', 'Eiche', 'Palivové dřevo', 'Brennholz', NULL, NULL, NULL,
   NULL, NULL, 'firewood', 'palivove', 'hard', 1, 1, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (3, 'fosny', 'Fošny', NULL, 'Stavební řezivo', 'Bauholz', NULL, 'Fošny na míru.', NULL,
   NULL, NULL, 'lumber', 'rezivo', NULL, 2, 1, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (4, 'stepka', 'Štěpka', 'Hackschnitzel', 'Výrobní zbytky', NULL, NULL, NULL, NULL,
   NULL, NULL, 'simple_variant', 'zbytky', NULL, 3, 1, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (5, 'skryty', 'Skrytý', NULL, NULL, NULL, NULL, NULL, NULL,
   NULL, NULL, 'simple_variant', 'zbytky', NULL, 4, 0, '2026-01-01 10:00:00', '2026-01-01 10:00:00');

INSERT INTO catalog_variants (id, catalog_product_id, key, variant_group, length_label, amount_value, price_czk, in_stock, stacked,
  grade, grade_de, width_mm, height_mm, variant_label, variant_label_de, position, created_at, updated_at) VALUES
  (10, 1, 'k-25-5', 'kontejner', '25 cm', 5, 9000, 1, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (11, 1, 'k-25-14', 'kontejner', '25 cm', 14, 21500, 1, 0, NULL, NULL, NULL, NULL, NULL, NULL, 1, '2026-01-01 10:00:00', '2026-03-05 08:30:00'),
  (12, 1, 'k-50-5', 'kontejner', '50 cm', 5, 9500, 1, 1, NULL, NULL, NULL, NULL, NULL, NULL, 2, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (13, 1, 'k-50-13', 'kontejner', '50 cm', 13, 20000, 0, 1, NULL, NULL, NULL, NULL, NULL, NULL, 3, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (14, 1, 'b-50-1', 'bedny', '50 cm', 1, 2500, 1, 0, NULL, NULL, NULL, NULL, NULL, NULL, 4, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (20, 2, 'k-33-5', 'kontejner', '33 cm', 5, 11000, 1, 0, NULL, NULL, NULL, NULL, NULL, NULL, 0, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (30, 3, 'f-ab-4', NULL, '4 m', NULL, 8900, 1, 0, 'Jakost AB', 'Qualität AB', 200, 50, NULL, NULL, 0, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (31, 3, 'f-ab-5', NULL, '5 m', NULL, 9900, 1, 0, 'Jakost AB', 'Qualität AB', 240, 60, NULL, NULL, 1, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (40, 4, 's-1', NULL, NULL, NULL, 1200, 1, 0, NULL, NULL, NULL, NULL, 'Kontejner 5 m³', 'Container 5 m³', 0, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (41, 4, 's-2', NULL, NULL, NULL, 2000, 0, 0, NULL, NULL, NULL, NULL, 'Kontejner 10 m³', NULL, 1, '2026-01-01 10:00:00', '2026-01-01 10:00:00');

INSERT INTO active_storage_blobs (id, key, filename, content_type, metadata, service_name, byte_size, checksum, created_at) VALUES
  (1, 'testkey0000000000000000000001', 'smrk foto.webp', 'image/webp', '{}', 'local', 16, 'x', '2026-01-01 10:00:00');
INSERT INTO active_storage_attachments (id, name, record_type, record_id, blob_id, created_at) VALUES
  (1, 'image', 'CatalogProduct', 1, 1, '2026-01-01 10:00:00');

INSERT INTO promos (id, title, title_de, feature_1, feature_2, feature_3, price, price_de, link, created_at, updated_at) VALUES
  (1, 'Akční smrk', 'Fichte im Angebot', 'Suché', 'Levné', 'Hned', 'od 8 500 Kč', NULL, '/eshop/smrk', '2026-01-01 10:00:00', '2026-01-01 10:00:00');

INSERT INTO services (id, title, title_de, content, content_de, button_text, button_text_de, button_path, images, created_at, updated_at) VALUES
  (1, 'Cenová kalkulace', NULL, 'Nezávazná kalkulace.', NULL, 'Poptat', NULL, '/kontakt#kalkulace', NULL, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (2, 'Doprava', 'Lieferung', 'Dovezeme.<br /><br />Vlastním vozem.', 'Wir liefern.', 'Ceník', 'Preisliste', '/sluzby#doprava', '["sluzby/doprava1.webp"]', '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (3, 'Pořez kulatiny', NULL, 'Pořežeme.', NULL, NULL, NULL, '/sluzby', '[]', '2026-01-01 10:00:00', '2026-01-01 10:00:00');

INSERT INTO products (id, title, title_de, text, text_de, image, link, created_at, updated_at) VALUES
  (1, 'Palivové dřevo', 'Brennholz', 'Suché dřevo', 'Trockenes Holz', 'sortiment/palivove-drevo1.webp', '/sortiment/palivove-drevo', '2026-01-01 10:00:00', '2026-01-01 10:00:00');

INSERT INTO faq_items (id, title, title_de, content, content_de, image, created_at, updated_at) VALUES
  (1, 'Jak skladovat dřevo', NULL, 'Pod <strong>střechou</strong>.', NULL, 'co-je-noveho-pic1.webp', '2026-01-01 10:00:00', '2026-01-01 10:00:00');

INSERT INTO reviews (id, name, avatar, stars, text, reviewed_on, position, created_at, updated_at) VALUES
  (1, 'jana', NULL, 5, 'Skvělé dřevo <3', '2025-01-10', 0, '2026-01-01 10:00:00', '2026-01-01 10:00:00');

INSERT INTO pricelist_items (id, category, subcategory, subcategory_de, item_name, item_name_de, details, details_de, price, price_de, position, created_at, updated_at) VALUES
  (1, 'stavebni', NULL, NULL, 'Hranoly', 'Kanthölzer', 'do 6 m', NULL, '9 800 Kč / m<sup>3</sup>', NULL, 0, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (2, 'palivove_volne', 'Měkké', 'Weichholz', '25 cm', NULL, NULL, NULL, '1 800 Kč', '75 €', 0, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (3, 'palivove_volne', 'Měkké', 'Weichholz', '33 cm', NULL, NULL, NULL, '---', NULL, 1, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (4, 'kamenivo', NULL, NULL, NULL, NULL, NULL, NULL, '<script>x</script>1 500 Kč', NULL, 0, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (5, 'sluzby', NULL, NULL, 'Hoblování', 'Hobeln', NULL, NULL, '500 Kč / hod', NULL, 0, '2026-01-01 10:00:00', '2026-01-01 10:00:00');

INSERT INTO inquiry_form_options (id, category, field, value, value_de, position, created_at, updated_at) VALUES
  (1, 'palivove', 'druh', 'Buk', 'Buche', 0, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (2, 'palivove', 'druh', 'Dub', NULL, 1, '2026-01-01 10:00:00', '2026-01-01 10:00:00'),
  (3, 'stavebni', 'polozka', 'Trámy', 'Balken', 0, '2026-01-01 10:00:00', '2026-01-01 10:00:00');
