-- The tables of the site. Every later change is a new file in this folder
-- (see README.md); never edit this one once a database has it.

-- Uploaded files (product and promo photos, profile pictures). The file is
-- storage/uploads/<key>; a record points at its upload with image_id or
-- avatar_id.
CREATE TABLE uploads (
  id INTEGER PRIMARY KEY,
  key TEXT NOT NULL UNIQUE,
  filename TEXT NOT NULL,
  content_type TEXT NOT NULL,
  byte_size INTEGER NOT NULL,
  created_at TEXT NOT NULL
);

-- The e-shop: products and the variants you can buy.
CREATE TABLE catalog_products (
  id INTEGER PRIMARY KEY,
  key TEXT NOT NULL UNIQUE,
  template TEXT NOT NULL,
  category TEXT NOT NULL,
  hardness TEXT,
  position INTEGER NOT NULL DEFAULT 0,
  active BOOLEAN NOT NULL DEFAULT TRUE,
  title TEXT NOT NULL,
  title_de TEXT,
  type_label TEXT,
  type_label_de TEXT,
  subtitle TEXT,
  subtitle_de TEXT,
  description TEXT,
  description_de TEXT,
  drying_note TEXT,
  drying_note_de TEXT,
  image_id INTEGER REFERENCES uploads (id),
  image_alt TEXT,
  image_alt_de TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE catalog_variants (
  id INTEGER PRIMARY KEY,
  catalog_product_id INTEGER NOT NULL REFERENCES catalog_products (id),
  key TEXT NOT NULL,
  position INTEGER NOT NULL DEFAULT 0,
  variant_group TEXT,
  length_label TEXT,
  amount_value INTEGER,
  price_czk INTEGER NOT NULL,
  in_stock BOOLEAN NOT NULL DEFAULT TRUE,
  stacked BOOLEAN NOT NULL DEFAULT FALSE,
  grade TEXT,
  grade_de TEXT,
  width_mm INTEGER,
  height_mm INTEGER,
  variant_label TEXT,
  variant_label_de TEXT,
  image_id INTEGER REFERENCES uploads (id),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (catalog_product_id, key)
);

-- The weekly promo on the home page.
CREATE TABLE promos (
  id INTEGER PRIMARY KEY,
  title TEXT,
  title_de TEXT,
  feature_1 TEXT,
  feature_1_de TEXT,
  feature_2 TEXT,
  feature_2_de TEXT,
  feature_3 TEXT,
  feature_3_de TEXT,
  price TEXT,
  price_de TEXT,
  link TEXT,
  image_id INTEGER REFERENCES uploads (id),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- Content of the pages: the price list, the quote form's choices, the
-- category tiles, services, "did you know", Google reviews.
CREATE TABLE pricelist_items (
  id INTEGER PRIMARY KEY,
  category TEXT,
  subcategory TEXT,
  subcategory_de TEXT,
  item_name TEXT,
  item_name_de TEXT,
  details TEXT,
  details_de TEXT,
  price TEXT,
  price_de TEXT,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX pricelist_items_by_category ON pricelist_items (category, position);

CREATE TABLE inquiry_form_options (
  id INTEGER PRIMARY KEY,
  category TEXT NOT NULL,
  field TEXT NOT NULL,
  value TEXT NOT NULL,
  value_de TEXT,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX inquiry_form_options_by_field ON inquiry_form_options (category, field, position);

CREATE TABLE products (
  id INTEGER PRIMARY KEY,
  title TEXT,
  title_de TEXT,
  text TEXT,
  text_de TEXT,
  image TEXT,
  link TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE services (
  id INTEGER PRIMARY KEY,
  title TEXT,
  title_de TEXT,
  content TEXT,
  content_de TEXT,
  button_text TEXT,
  button_text_de TEXT,
  button_path TEXT,
  images TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE faq_items (
  id INTEGER PRIMARY KEY,
  title TEXT,
  title_de TEXT,
  content TEXT,
  content_de TEXT,
  image TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE reviews (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  avatar TEXT,
  stars INTEGER NOT NULL DEFAULT 5,
  text TEXT NOT NULL,
  reviewed_on TEXT NOT NULL,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX reviews_by_position ON reviews (position);

-- Customer accounts, and their logins (one row per signed-in browser; the
-- login cookie holds its id).
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  email_address TEXT NOT NULL UNIQUE,
  password_digest TEXT NOT NULL,
  role TEXT NOT NULL DEFAULT 'customer',
  first_name TEXT,
  last_name TEXT,
  phone TEXT,
  company_purchase BOOLEAN NOT NULL DEFAULT FALSE,
  company_name TEXT,
  company_ico TEXT,
  company_dic TEXT,
  billing_street TEXT,
  billing_city TEXT,
  billing_zip TEXT,
  billing_country TEXT,
  delivery_address_different BOOLEAN NOT NULL DEFAULT FALSE,
  delivery_street TEXT,
  delivery_city TEXT,
  delivery_zip TEXT,
  delivery_country TEXT,
  avatar_id INTEGER REFERENCES uploads (id),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE sessions (
  id INTEGER PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users (id),
  ip_address TEXT,
  user_agent TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX sessions_by_user ON sessions (user_id);

-- Orders and quote requests. items_snapshot is the cart or the requested
-- items as JSON, frozen when sent.
CREATE TABLE orders (
  -- AUTOINCREMENT: an order number is never given out twice.
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER REFERENCES users (id),
  locale TEXT NOT NULL DEFAULT 'cs',
  items_snapshot TEXT NOT NULL,
  subtotal_czk INTEGER NOT NULL,
  vat_czk INTEGER NOT NULL,
  total_czk INTEGER NOT NULL,
  shipping_method TEXT,
  delivery_distance_km INTEGER,
  delivery_czk INTEGER,
  payment_method TEXT,
  first_name TEXT,
  last_name TEXT,
  email TEXT,
  phone TEXT,
  company_purchase BOOLEAN NOT NULL DEFAULT FALSE,
  company_name TEXT,
  company_ico TEXT,
  company_dic TEXT,
  billing_street TEXT,
  billing_city TEXT,
  billing_zip TEXT,
  billing_country TEXT,
  delivery_address_different BOOLEAN NOT NULL DEFAULT FALSE,
  delivery_street TEXT,
  delivery_city TEXT,
  delivery_zip TEXT,
  delivery_country TEXT,
  newsletter_opt_in BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX orders_by_user ON orders (user_id);

CREATE TABLE inquiries (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  category TEXT NOT NULL DEFAULT 'palivove',
  locale TEXT NOT NULL DEFAULT 'cs',
  items_snapshot TEXT NOT NULL,
  first_name TEXT,
  last_name TEXT,
  email TEXT,
  phone TEXT,
  street TEXT,
  house_number TEXT,
  city TEXT,
  zip TEXT,
  notes TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
