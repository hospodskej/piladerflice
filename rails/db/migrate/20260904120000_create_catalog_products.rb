class CreateCatalogProducts < ActiveRecord::Migration[8.1]
  def change
    create_table :catalog_products do |t|
      t.string :key, null: false

      t.string :template, null: false

      t.string :category, null: false

      t.string :hardness

      t.string :image
      t.boolean :active, default: true, null: false
      t.integer :position, default: 0, null: false

      t.string :title, null: false
      t.string :title_de
      t.string :type_label
      t.string :type_label_de
      t.string :subtitle
      t.string :subtitle_de
      t.text :description
      t.text :description_de
      t.string :drying_note
      t.string :drying_note_de
      t.string :image_alt
      t.string :image_alt_de

      t.timestamps
    end
    add_index :catalog_products, :key, unique: true

    create_table :catalog_variants do |t|
      t.references :catalog_product, null: false, foreign_key: true

      t.string :key, null: false

      t.string :variant_group

      t.string :length_label

      t.string :variant_label
      t.string :variant_label_de

      t.integer :amount_value

      t.integer :price_czk, null: false
      t.boolean :in_stock, default: true, null: false
      t.integer :position, default: 0, null: false

      t.timestamps
    end
    add_index :catalog_variants, [:catalog_product_id, :key], unique: true
  end
end
