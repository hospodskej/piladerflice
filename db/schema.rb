# This file is auto-generated from the current state of the database. Instead
# of editing this file, please use the migrations feature of Active Record to
# incrementally modify your database, and then regenerate this schema definition.
#
# This file is the source Rails uses to define your schema when running `bin/rails
# db:schema:load`. When creating a new database, `bin/rails db:schema:load` tends to
# be faster and is potentially less error prone than running all of your
# migrations from scratch. Old migrations may fail to apply correctly if those
# migrations use external dependencies or application code.
#
# It's strongly recommended that you check this file into your version control system.

ActiveRecord::Schema[8.1].define(version: 2026_09_14_120001) do
  create_table "catalog_products", force: :cascade do |t|
    t.boolean "active", default: true, null: false
    t.string "category", null: false
    t.datetime "created_at", null: false
    t.text "description"
    t.text "description_de"
    t.string "drying_note"
    t.string "drying_note_de"
    t.string "hardness"
    t.string "image"
    t.string "image_alt"
    t.string "image_alt_de"
    t.string "key", null: false
    t.integer "position", default: 0, null: false
    t.string "subtitle"
    t.string "subtitle_de"
    t.string "template", null: false
    t.string "title", null: false
    t.string "title_de"
    t.string "type_label"
    t.string "type_label_de"
    t.datetime "updated_at", null: false
    t.index ["key"], name: "index_catalog_products_on_key", unique: true
  end

  create_table "catalog_variants", force: :cascade do |t|
    t.integer "amount_value"
    t.integer "catalog_product_id", null: false
    t.datetime "created_at", null: false
    t.boolean "in_stock", default: true, null: false
    t.string "key", null: false
    t.string "length_label"
    t.integer "position", default: 0, null: false
    t.integer "price_czk", null: false
    t.datetime "updated_at", null: false
    t.string "variant_group"
    t.string "variant_label"
    t.string "variant_label_de"
    t.index ["catalog_product_id", "key"], name: "index_catalog_variants_on_catalog_product_id_and_key", unique: true
    t.index ["catalog_product_id"], name: "index_catalog_variants_on_catalog_product_id"
  end

  create_table "faq_items", force: :cascade do |t|
    t.text "content"
    t.text "content_de"
    t.datetime "created_at", null: false
    t.string "image"
    t.string "title"
    t.string "title_de"
    t.datetime "updated_at", null: false
  end

  create_table "inquiries", force: :cascade do |t|
    t.string "category", default: "palivove", null: false
    t.string "city"
    t.datetime "created_at", null: false
    t.string "email"
    t.string "first_name"
    t.string "house_number"
    t.text "items_snapshot", null: false
    t.string "last_name"
    t.string "locale", default: "cs", null: false
    t.text "notes"
    t.string "phone"
    t.string "street"
    t.datetime "updated_at", null: false
    t.string "zip"
  end

  create_table "inquiry_form_options", force: :cascade do |t|
    t.string "category", null: false
    t.datetime "created_at", null: false
    t.string "field", null: false
    t.integer "position", default: 0, null: false
    t.datetime "updated_at", null: false
    t.string "value", null: false
    t.string "value_de"
    t.index ["category", "field", "position"], name: "index_inquiry_form_options_on_category_and_field_and_position"
  end

  create_table "orders", force: :cascade do |t|
    t.string "billing_city"
    t.string "billing_country"
    t.string "billing_street"
    t.string "billing_zip"
    t.string "company_dic"
    t.string "company_ico"
    t.string "company_name"
    t.boolean "company_purchase", default: false, null: false
    t.datetime "created_at", null: false
    t.boolean "delivery_address_different", default: false, null: false
    t.string "delivery_city"
    t.string "delivery_country"
    t.string "delivery_street"
    t.string "delivery_zip"
    t.string "email"
    t.string "first_name"
    t.text "items_snapshot", null: false
    t.string "last_name"
    t.string "locale", default: "cs", null: false
    t.boolean "newsletter_opt_in", default: false, null: false
    t.string "payment_method"
    t.string "phone"
    t.string "shipping_method"
    t.integer "subtotal_czk", null: false
    t.integer "total_czk", null: false
    t.datetime "updated_at", null: false
    t.integer "vat_czk", null: false
  end

  create_table "pricelist_items", force: :cascade do |t|
    t.string "category"
    t.datetime "created_at", null: false
    t.string "details"
    t.string "details_de"
    t.string "item_name"
    t.string "item_name_de"
    t.string "price"
    t.string "price_de"
    t.string "subcategory"
    t.string "subcategory_de"
    t.datetime "updated_at", null: false
  end

  create_table "products", force: :cascade do |t|
    t.datetime "created_at", null: false
    t.string "image"
    t.string "link"
    t.text "text"
    t.text "text_de"
    t.string "title"
    t.string "title_de"
    t.datetime "updated_at", null: false
  end

  create_table "promos", force: :cascade do |t|
    t.datetime "created_at", null: false
    t.string "feature_1"
    t.string "feature_1_de"
    t.string "feature_2"
    t.string "feature_2_de"
    t.string "feature_3"
    t.string "feature_3_de"
    t.string "image"
    t.string "link"
    t.string "price"
    t.string "price_de"
    t.string "title"
    t.string "title_de"
    t.datetime "updated_at", null: false
  end

  create_table "services", force: :cascade do |t|
    t.string "button_path"
    t.string "button_text"
    t.string "button_text_de"
    t.text "content"
    t.text "content_de"
    t.datetime "created_at", null: false
    t.text "images"
    t.string "title"
    t.string "title_de"
    t.datetime "updated_at", null: false
  end

  create_table "sessions", force: :cascade do |t|
    t.datetime "created_at", null: false
    t.string "ip_address"
    t.datetime "updated_at", null: false
    t.string "user_agent"
    t.integer "user_id", null: false
    t.index ["user_id"], name: "index_sessions_on_user_id"
  end

  create_table "users", force: :cascade do |t|
    t.datetime "created_at", null: false
    t.string "email_address", null: false
    t.string "password_digest", null: false
    t.string "role", default: "admin", null: false
    t.datetime "updated_at", null: false
    t.index ["email_address"], name: "index_users_on_email_address", unique: true
  end

  add_foreign_key "catalog_variants", "catalog_products"
  add_foreign_key "sessions", "users"
end
