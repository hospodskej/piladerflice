-- The database schema, exported from the Rails app's SQLite database
-- (db/schema.rb). The Rust tests build a fresh database from it.
-- Regenerate it when a Rails migration changes the schema.

CREATE TABLE "active_storage_attachments" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "blob_id" bigint NOT NULL, "created_at" datetime(6) NOT NULL, "name" varchar NOT NULL, "record_id" bigint NOT NULL, "record_type" varchar NOT NULL, CONSTRAINT "fk_rails_c3b3935057"
FOREIGN KEY ("blob_id")
  REFERENCES "active_storage_blobs" ("id")
);
CREATE TABLE "active_storage_blobs" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "byte_size" bigint NOT NULL, "checksum" varchar, "content_type" varchar, "created_at" datetime(6) NOT NULL, "filename" varchar NOT NULL, "key" varchar NOT NULL, "metadata" text, "service_name" varchar NOT NULL);
CREATE TABLE "active_storage_variant_records" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "blob_id" bigint NOT NULL, "variation_digest" varchar NOT NULL, CONSTRAINT "fk_rails_993965df05"
FOREIGN KEY ("blob_id")
  REFERENCES "active_storage_blobs" ("id")
);
CREATE TABLE "ar_internal_metadata" ("key" varchar NOT NULL PRIMARY KEY, "value" varchar, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE "catalog_products" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "active" boolean DEFAULT TRUE NOT NULL, "category" varchar NOT NULL, "created_at" datetime(6) NOT NULL, "description" text, "description_de" text, "drying_note" varchar, "drying_note_de" varchar, "hardness" varchar, "image_alt" varchar, "image_alt_de" varchar, "key" varchar NOT NULL, "position" integer DEFAULT 0 NOT NULL, "subtitle" varchar, "subtitle_de" varchar, "template" varchar NOT NULL, "title" varchar NOT NULL, "title_de" varchar, "type_label" varchar, "type_label_de" varchar, "updated_at" datetime(6) NOT NULL);
CREATE TABLE "catalog_variants" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "amount_value" integer, "catalog_product_id" integer NOT NULL, "created_at" datetime(6) NOT NULL, "grade" varchar, "grade_de" varchar, "height_mm" integer, "in_stock" boolean DEFAULT TRUE NOT NULL, "key" varchar NOT NULL, "length_label" varchar, "position" integer DEFAULT 0 NOT NULL, "price_czk" integer NOT NULL, "stacked" boolean DEFAULT FALSE NOT NULL, "updated_at" datetime(6) NOT NULL, "variant_group" varchar, "variant_label" varchar, "variant_label_de" varchar, "width_mm" integer, CONSTRAINT "fk_rails_d5110426e6"
FOREIGN KEY ("catalog_product_id")
  REFERENCES "catalog_products" ("id")
);
CREATE TABLE "faq_items" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "content" text, "content_de" text, "created_at" datetime(6) NOT NULL, "image" varchar, "title" varchar, "title_de" varchar, "updated_at" datetime(6) NOT NULL);
CREATE TABLE "inquiries" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "category" varchar DEFAULT 'palivove' NOT NULL, "city" varchar, "created_at" datetime(6) NOT NULL, "email" varchar, "first_name" varchar, "house_number" varchar, "items_snapshot" text NOT NULL, "last_name" varchar, "locale" varchar DEFAULT 'cs' NOT NULL, "notes" text, "phone" varchar, "street" varchar, "updated_at" datetime(6) NOT NULL, "zip" varchar);
CREATE TABLE "inquiry_form_options" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "category" varchar NOT NULL, "created_at" datetime(6) NOT NULL, "field" varchar NOT NULL, "position" integer DEFAULT 0 NOT NULL, "updated_at" datetime(6) NOT NULL, "value" varchar NOT NULL, "value_de" varchar);
CREATE TABLE "orders" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "billing_city" varchar, "billing_country" varchar, "billing_street" varchar, "billing_zip" varchar, "company_dic" varchar, "company_ico" varchar, "company_name" varchar, "company_purchase" boolean DEFAULT FALSE NOT NULL, "created_at" datetime(6) NOT NULL, "delivery_address_different" boolean DEFAULT FALSE NOT NULL, "delivery_city" varchar, "delivery_country" varchar, "delivery_street" varchar, "delivery_zip" varchar, "email" varchar, "first_name" varchar, "items_snapshot" text NOT NULL, "last_name" varchar, "locale" varchar DEFAULT 'cs' NOT NULL, "newsletter_opt_in" boolean DEFAULT FALSE NOT NULL, "payment_method" varchar, "phone" varchar, "shipping_method" varchar, "subtotal_czk" integer NOT NULL, "total_czk" integer NOT NULL, "updated_at" datetime(6) NOT NULL, "user_id" integer, "vat_czk" integer NOT NULL, "delivery_distance_km" integer /*application='PilaDerflice'*/, "delivery_czk" integer /*application='PilaDerflice'*/, CONSTRAINT "fk_rails_f868b47f6a"
FOREIGN KEY ("user_id")
  REFERENCES "users" ("id")
);
CREATE TABLE "pricelist_items" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "category" varchar, "created_at" datetime(6) NOT NULL, "details" varchar, "details_de" varchar, "item_name" varchar, "item_name_de" varchar, "position" integer DEFAULT 0 NOT NULL, "price" varchar, "price_de" varchar, "subcategory" varchar, "subcategory_de" varchar, "updated_at" datetime(6) NOT NULL);
CREATE TABLE "products" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "image" varchar, "link" varchar, "text" text, "text_de" text, "title" varchar, "title_de" varchar, "updated_at" datetime(6) NOT NULL);
CREATE TABLE "promos" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "feature_1" varchar, "feature_1_de" varchar, "feature_2" varchar, "feature_2_de" varchar, "feature_3" varchar, "feature_3_de" varchar, "link" varchar, "price" varchar, "price_de" varchar, "title" varchar, "title_de" varchar, "updated_at" datetime(6) NOT NULL);
CREATE TABLE "reviews" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "avatar" varchar, "created_at" datetime(6) NOT NULL, "name" varchar NOT NULL, "position" integer DEFAULT 0 NOT NULL, "reviewed_on" date NOT NULL, "stars" integer DEFAULT 5 NOT NULL, "text" text NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE "schema_migrations" ("version" varchar NOT NULL PRIMARY KEY);
CREATE TABLE "services" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "button_path" varchar, "button_text" varchar, "button_text_de" varchar, "content" text, "content_de" text, "created_at" datetime(6) NOT NULL, "images" text, "title" varchar, "title_de" varchar, "updated_at" datetime(6) NOT NULL);
CREATE TABLE "sessions" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "ip_address" varchar, "updated_at" datetime(6) NOT NULL, "user_agent" varchar, "user_id" integer NOT NULL, CONSTRAINT "fk_rails_758836b4f0"
FOREIGN KEY ("user_id")
  REFERENCES "users" ("id")
);
CREATE TABLE "users" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "billing_city" varchar, "billing_country" varchar, "billing_street" varchar, "billing_zip" varchar, "company_dic" varchar, "company_ico" varchar, "company_name" varchar, "company_purchase" boolean DEFAULT FALSE NOT NULL, "created_at" datetime(6) NOT NULL, "delivery_address_different" boolean DEFAULT FALSE NOT NULL, "delivery_city" varchar, "delivery_country" varchar, "delivery_street" varchar, "delivery_zip" varchar, "email_address" varchar NOT NULL, "first_name" varchar, "last_name" varchar, "password_digest" varchar NOT NULL, "phone" varchar, "role" varchar DEFAULT 'customer' NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE INDEX "index_active_storage_attachments_on_blob_id" ON "active_storage_attachments" ("blob_id") /*application='PilaDerflice'*/;
CREATE UNIQUE INDEX "index_active_storage_attachments_uniqueness" ON "active_storage_attachments" ("record_type", "record_id", "name", "blob_id") /*application='PilaDerflice'*/;
CREATE UNIQUE INDEX "index_active_storage_blobs_on_key" ON "active_storage_blobs" ("key") /*application='PilaDerflice'*/;
CREATE UNIQUE INDEX "index_active_storage_variant_records_uniqueness" ON "active_storage_variant_records" ("blob_id", "variation_digest") /*application='PilaDerflice'*/;
CREATE UNIQUE INDEX "index_catalog_products_on_key" ON "catalog_products" ("key") /*application='PilaDerflice'*/;
CREATE INDEX "index_catalog_variants_on_catalog_product_id" ON "catalog_variants" ("catalog_product_id") /*application='PilaDerflice'*/;
CREATE UNIQUE INDEX "index_catalog_variants_on_catalog_product_id_and_key" ON "catalog_variants" ("catalog_product_id", "key") /*application='PilaDerflice'*/;
CREATE INDEX "index_inquiry_form_options_on_category_and_field_and_position" ON "inquiry_form_options" ("category", "field", "position") /*application='PilaDerflice'*/;
CREATE INDEX "index_orders_on_user_id" ON "orders" ("user_id") /*application='PilaDerflice'*/;
CREATE INDEX "index_pricelist_items_on_category_and_position" ON "pricelist_items" ("category", "position") /*application='PilaDerflice'*/;
CREATE INDEX "index_reviews_on_position" ON "reviews" ("position") /*application='PilaDerflice'*/;
CREATE INDEX "index_sessions_on_user_id" ON "sessions" ("user_id") /*application='PilaDerflice'*/;
CREATE UNIQUE INDEX "index_users_on_email_address" ON "users" ("email_address") /*application='PilaDerflice'*/;
