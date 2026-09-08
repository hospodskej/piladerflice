class CreateCatalogProducts < ActiveRecord::Migration[8.1]
  def change
    create_table :catalog_products do |t|
      # Canonical identifier, e.g. "smrk", "dub", "tramy", "odkory" - the
      # same keys already used throughout the cart/checkout system (see
      # CartLineItem#product_key), so switching the eshop pages over to
      # read from here doesn't touch anything downstream of the cart.
      t.string :key, null: false

      # Which shared view partial renders this product's detail page:
      # "firewood" (species pages with a container+crate matrix),
      # "lumber" (single dimension/length matrix), or "simple_variant"
      # (a flat grid of variants, no matrix) - see app/views/shared/.
      t.string :template, null: false

      # Matches the ?category= filter param on /eshop: "palivove",
      # "rezivo", "zbytky", or "kamenivo".
      t.string :category, null: false

      # "hard" / "soft" - only meaningful for firewood, used by the
      # eshop wood-type filter; nil for every other template.
      t.string :hardness

      t.string :image
      t.boolean :active, default: true, null: false
      t.integer :position, default: 0, null: false

      # Admin-editable catalog text, Czech + German. Not every field
      # applies to every template (e.g. only firewood uses subtitle/
      # drying_note) - unused fields are simply left blank for products
      # that don't need them.
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

      # Stable identifier within the product, e.g. "loose-100cm-15prm" -
      # not used for lookups (the numeric id is), but keeps variants
      # human-identifiable in the admin UI.
      t.string :key, null: false

      # "kontejner" / "bedny" / nil - which section of the firewood
      # template's two-part layout this variant belongs to (determines
      # whether it displays as "Sypané"/"Lose" or "Skládané"/"Gestapelt" -
      # see CatalogVariant#to_cart_specs). Always nil for every other
      # template.
      t.string :variant_group

      # The matrix "column" value where applicable - a length/dimension
      # like "100 cm" or "3000 mm". Language-neutral (measurements read
      # the same in Czech and German), so no _de column needed. Left
      # blank for the simple_variant template, which has no matrix.
      t.string :length_label

      # The variant's own label - a wood species name ("Smrk"), a
      # packaging option ("Volně ložené"), a dimension ("40/50 mm"), or a
      # quality grade ("I. netříděné") depending on template. Admin-
      # editable in both languages directly (unlike the rest of the
      # site's static UI text, this is admin content, not translated via
      # the locale YAML files) - see CatalogVariant#to_cart_specs and the
      # new "bilingual" spec type in CartHelper.
      t.string :variant_label
      t.string :variant_label_de

      # Firewood only - the PRM amount (15, 10, 5, or 1 for a crate).
      t.integer :amount_value

      t.integer :price_czk, null: false
      t.boolean :in_stock, default: true, null: false
      t.integer :position, default: 0, null: false

      t.timestamps
    end
    add_index :catalog_variants, [:catalog_product_id, :key], unique: true
  end
end
