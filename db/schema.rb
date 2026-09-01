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

ActiveRecord::Schema[8.1].define(version: 2026_08_28_120000) do
  create_table "faq_items", force: :cascade do |t|
    t.text "content"
    t.datetime "created_at", null: false
    t.string "image"
    t.string "title"
    t.datetime "updated_at", null: false
    t.string "title_de"
    t.text "content_de"
  end

  create_table "orders", force: :cascade do |t|
    t.string "first_name"
    t.string "last_name"
    t.string "email"
    t.string "phone"
    t.boolean "newsletter_opt_in", default: false, null: false
    t.string "billing_street"
    t.string "billing_city"
    t.string "billing_zip"
    t.string "billing_country"
    t.boolean "company_purchase", default: false, null: false
    t.string "company_name"
    t.string "company_ico"
    t.string "company_dic"
    t.boolean "delivery_address_different", default: false, null: false
    t.string "delivery_street"
    t.string "delivery_city"
    t.string "delivery_zip"
    t.string "delivery_country"
    t.string "shipping_method"
    t.string "payment_method"
    t.text "items_snapshot", null: false
    t.integer "subtotal_czk", null: false
    t.integer "vat_czk", null: false
    t.integer "total_czk", null: false
    t.string "locale", default: "cs", null: false
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
  end

  create_table "pricelist_items", force: :cascade do |t|
    t.string "category"
    t.datetime "created_at", null: false
    t.string "details"
    t.string "item_name"
    t.string "price"
    t.string "subcategory"
    t.datetime "updated_at", null: false
    t.string "item_name_de"
    t.string "details_de"
    t.string "price_de"
    t.string "subcategory_de"
  end

  create_table "products", force: :cascade do |t|
    t.datetime "created_at", null: false
    t.string "image"
    t.string "link"
    t.text "text"
    t.string "title"
    t.datetime "updated_at", null: false
    t.string "title_de"
    t.text "text_de"
  end

  create_table "promos", force: :cascade do |t|
    t.datetime "created_at", null: false
    t.string "feature_1"
    t.string "feature_2"
    t.string "feature_3"
    t.string "image"
    t.string "link"
    t.string "price"
    t.string "title"
    t.datetime "updated_at", null: false
    t.string "title_de"
    t.string "feature_1_de"
    t.string "feature_2_de"
    t.string "feature_3_de"
    t.string "price_de"
  end

  create_table "services", force: :cascade do |t|
    t.string "button_path"
    t.string "button_text"
    t.text "content"
    t.datetime "created_at", null: false
    t.text "images"
    t.string "title"
    t.datetime "updated_at", null: false
    t.string "title_de"
    t.text "content_de"
    t.string "button_text_de"
  end
end
