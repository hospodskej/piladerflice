-- The whole Pila Derflice schema. Values that have a fixed set of options are
-- checked by the database too, so a bad value can't be stored even by hand.
-- Timestamps are ISO 8601 text in UTC. Prices are whole crowns (CZK).
-- Pictures are stored as a site path: "/img/..." for the site's own pictures,
-- "/uploads/<hash>.webp" for photos uploaded in the admin.

CREATE TABLE users (
    id                         INTEGER PRIMARY KEY,
    email_address              TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    password_hash              TEXT    NOT NULL,
    role                       TEXT    NOT NULL DEFAULT 'customer' CHECK (role IN ('admin', 'customer')),
    -- Raised when the password changes: every signed-in browser then signs out.
    session_epoch              INTEGER NOT NULL DEFAULT 0,
    theme                      TEXT    CHECK (theme IN ('light', 'dark')),
    avatar_path                TEXT,
    first_name                 TEXT,
    last_name                  TEXT,
    phone                      TEXT,
    billing_street             TEXT,
    billing_city               TEXT,
    billing_zip                TEXT,
    billing_country            TEXT,
    company_purchase           INTEGER NOT NULL DEFAULT 0 CHECK (company_purchase IN (0, 1)),
    company_name               TEXT,
    company_ico                TEXT,
    company_dic                TEXT,
    delivery_address_different INTEGER NOT NULL DEFAULT 0 CHECK (delivery_address_different IN (0, 1)),
    delivery_street            TEXT,
    delivery_city              TEXT,
    delivery_zip               TEXT,
    delivery_country           TEXT,
    created_at                 TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at                 TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Sortiment tiles on the home and sortiment pages.
CREATE TABLE products (
    id         INTEGER PRIMARY KEY,
    title      TEXT NOT NULL,
    title_de   TEXT,
    text       TEXT,
    text_de    TEXT,
    image      TEXT,
    link       TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE catalog_products (
    id             INTEGER PRIMARY KEY,
    key            TEXT    NOT NULL UNIQUE,
    template       TEXT    NOT NULL CHECK (template IN ('firewood', 'lumber', 'simple_variant')),
    category       TEXT    NOT NULL CHECK (category IN ('palivove', 'rezivo', 'zbytky', 'kamenivo')),
    hardness       TEXT    CHECK (hardness IN ('hard', 'soft')),
    active         INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    position       INTEGER NOT NULL DEFAULT 0,
    title          TEXT    NOT NULL,
    title_de       TEXT,
    type_label     TEXT,
    type_label_de  TEXT,
    subtitle       TEXT,
    subtitle_de    TEXT,
    description    TEXT,
    description_de TEXT,
    drying_note    TEXT,
    drying_note_de TEXT,
    image          TEXT,
    image_alt      TEXT,
    image_alt_de   TEXT,
    created_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE catalog_variants (
    id                 INTEGER PRIMARY KEY,
    catalog_product_id INTEGER NOT NULL REFERENCES catalog_products (id) ON DELETE CASCADE,
    key                TEXT    NOT NULL,
    position           INTEGER NOT NULL DEFAULT 0,
    -- Without VAT or with it, depending on the category's VAT rule.
    price_czk          INTEGER NOT NULL CHECK (price_czk > 0),
    in_stock           INTEGER NOT NULL DEFAULT 1 CHECK (in_stock IN (0, 1)),
    stacked            INTEGER NOT NULL DEFAULT 0 CHECK (stacked IN (0, 1)),
    variant_group      TEXT    CHECK (variant_group IN ('kontejner', 'bedny')),
    variant_label      TEXT,
    variant_label_de   TEXT,
    grade              TEXT,
    grade_de           TEXT,
    length_label       TEXT,
    amount_value       INTEGER,
    width_mm           INTEGER,
    height_mm          INTEGER,
    image              TEXT,
    created_at         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (catalog_product_id, key)
);

CREATE TABLE vat_rules (
    id                 INTEGER PRIMARY KEY,
    category           TEXT    NOT NULL UNIQUE CHECK (category IN ('palivove', 'rezivo', 'zbytky', 'kamenivo')),
    rate_cz            INTEGER NOT NULL DEFAULT 21 CHECK (rate_cz BETWEEN 0 AND 100),
    rate_at            INTEGER NOT NULL DEFAULT 20 CHECK (rate_at BETWEEN 0 AND 100),
    prices_include_vat INTEGER NOT NULL DEFAULT 1 CHECK (prices_include_vat IN (0, 1)),
    created_at         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE discount_codes (
    id            INTEGER PRIMARY KEY,
    code          TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    kind          TEXT    NOT NULL DEFAULT 'percent' CHECK (kind IN ('percent', 'fixed')),
    value         INTEGER NOT NULL CHECK (value > 0 AND (kind = 'fixed' OR value <= 100)),
    active        INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    valid_until   TEXT,
    usage_limit   INTEGER CHECK (usage_limit > 0),
    times_used    INTEGER NOT NULL DEFAULT 0,
    min_order_czk INTEGER CHECK (min_order_czk > 0),
    note          TEXT,
    created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE orders (
    id                         INTEGER PRIMARY KEY,
    user_id                    INTEGER REFERENCES users (id) ON DELETE SET NULL,
    locale                     TEXT    NOT NULL DEFAULT 'cs' CHECK (locale IN ('cs', 'de')),
    -- The cart lines as they were when the order was placed (JSON array).
    items                      TEXT    NOT NULL,
    subtotal_czk               INTEGER NOT NULL,
    discount_code              TEXT,
    discount_czk               INTEGER,
    delivery_distance_km       INTEGER,
    delivery_czk               INTEGER,
    vat_czk                    INTEGER NOT NULL,
    -- {"21": 1234, "13": 56}: VAT per rate.
    vat_breakdown              TEXT,
    vat_country                TEXT    CHECK (vat_country IN ('cz', 'at')),
    total_czk                  INTEGER NOT NULL,
    shipping_method            TEXT    NOT NULL CHECK (shipping_method IN ('address', 'pickup')),
    payment_method             TEXT    NOT NULL CHECK (payment_method IN ('cash', 'bank_transfer')),
    first_name                 TEXT    NOT NULL,
    last_name                  TEXT    NOT NULL,
    email                      TEXT    NOT NULL,
    phone                      TEXT    NOT NULL,
    billing_street             TEXT    NOT NULL,
    billing_city               TEXT    NOT NULL,
    billing_zip                TEXT    NOT NULL,
    billing_country            TEXT    NOT NULL,
    company_purchase           INTEGER NOT NULL DEFAULT 0 CHECK (company_purchase IN (0, 1)),
    company_name               TEXT,
    company_ico                TEXT,
    company_dic                TEXT,
    delivery_address_different INTEGER NOT NULL DEFAULT 0 CHECK (delivery_address_different IN (0, 1)),
    delivery_street            TEXT,
    delivery_city              TEXT,
    delivery_zip               TEXT,
    delivery_country           TEXT,
    newsletter_opt_in          INTEGER NOT NULL DEFAULT 0 CHECK (newsletter_opt_in IN (0, 1)),
    created_at                 TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at                 TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX orders_user_id ON orders (user_id);
CREATE INDEX orders_created_at ON orders (created_at);

CREATE TABLE inquiries (
    id           INTEGER PRIMARY KEY,
    locale       TEXT NOT NULL DEFAULT 'cs' CHECK (locale IN ('cs', 'de')),
    -- {"palivove": [{...}], "stavebni": [{...}]}: the rows of the inquiry form.
    items        TEXT NOT NULL,
    first_name   TEXT NOT NULL,
    last_name    TEXT NOT NULL,
    email        TEXT NOT NULL,
    phone        TEXT NOT NULL,
    street       TEXT,
    house_number TEXT,
    city         TEXT,
    zip          TEXT,
    notes        TEXT,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE inquiry_form_options (
    id         INTEGER PRIMARY KEY,
    category   TEXT    NOT NULL CHECK (category IN ('palivove', 'stavebni')),
    field      TEXT    NOT NULL CHECK (
                   (category = 'palivove' AND field IN ('varianta', 'druh', 'delka', 'mnozstvi'))
                OR (category = 'stavebni' AND field IN ('polozka', 'delka', 'vyska', 'sirka'))),
    position   INTEGER NOT NULL DEFAULT 0,
    value      TEXT    NOT NULL,
    value_de   TEXT,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX inquiry_form_options_order ON inquiry_form_options (category, field, position);

CREATE TABLE pricelist_items (
    id             INTEGER PRIMARY KEY,
    category       TEXT    NOT NULL CHECK (category IN ('stavebni', 'palivove_volne', 'palivove_skladane', 'zbytky', 'kamenivo', 'sluzby')),
    position       INTEGER NOT NULL DEFAULT 0,
    -- The row shows the admin's price per km instead of `price`.
    delivery_rate  INTEGER NOT NULL DEFAULT 0 CHECK (delivery_rate IN (0, 1)),
    subcategory    TEXT,
    subcategory_de TEXT,
    item_name      TEXT,
    item_name_de   TEXT,
    details        TEXT,
    details_de     TEXT,
    price          TEXT    NOT NULL,
    price_de       TEXT,
    created_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX pricelist_items_order ON pricelist_items (category, position);

CREATE TABLE promos (
    id           INTEGER PRIMARY KEY,
    title        TEXT,
    title_de     TEXT,
    feature_1    TEXT,
    feature_1_de TEXT,
    feature_2    TEXT,
    feature_2_de TEXT,
    feature_3    TEXT,
    feature_3_de TEXT,
    price        TEXT,
    price_de     TEXT,
    link         TEXT,
    image        TEXT,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE reviews (
    id          INTEGER PRIMARY KEY,
    name        TEXT    NOT NULL,
    text        TEXT    NOT NULL,
    stars       INTEGER NOT NULL DEFAULT 5 CHECK (stars BETWEEN 1 AND 5),
    reviewed_on TEXT    NOT NULL,
    avatar      TEXT,
    position    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX reviews_position ON reviews (position);

CREATE TABLE services (
    id             INTEGER PRIMARY KEY,
    title          TEXT,
    title_de       TEXT,
    content        TEXT,
    content_de     TEXT,
    -- JSON array of picture paths.
    images         TEXT NOT NULL DEFAULT '[]',
    button_text    TEXT,
    button_text_de TEXT,
    button_path    TEXT,
    created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE faq_items (
    id         INTEGER PRIMARY KEY,
    title      TEXT,
    title_de   TEXT,
    content    TEXT,
    content_de TEXT,
    image      TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Admin settings; the app knows the keys and their defaults.
CREATE TABLE settings (
    key        TEXT PRIMARY KEY CHECK (key IN ('delivery_czk_per_km')),
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Road distances from the sawmill, kept for 30 days per address or place.
CREATE TABLE route_cache (
    query      TEXT PRIMARY KEY,
    -- JSON: the place found and the one-way distance, or null when nothing was found.
    result     TEXT,
    expires_at TEXT NOT NULL
);
