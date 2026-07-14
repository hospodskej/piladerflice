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

ActiveRecord::Schema[8.1].define(version: 2026_07_14_133304) do
  create_table "faq_items", force: :cascade do |t|
    t.text "content"
    t.datetime "created_at", null: false
    t.string "image"
    t.string "title"
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
  end

  create_table "products", force: :cascade do |t|
    t.datetime "created_at", null: false
    t.string "image"
    t.string "link"
    t.text "text"
    t.string "title"
    t.datetime "updated_at", null: false
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
  end

  create_table "services", force: :cascade do |t|
    t.string "button_path"
    t.string "button_text"
    t.text "content"
    t.datetime "created_at", null: false
    t.text "images"
    t.string "title"
    t.datetime "updated_at", null: false
  end
end
