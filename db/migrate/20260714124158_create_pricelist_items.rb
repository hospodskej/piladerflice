class CreatePricelistItems < ActiveRecord::Migration[8.1]
  def change
    create_table :pricelist_items do |t|
      t.string :category
      t.string :subcategory
      t.string :item_name
      t.string :details
      t.string :price

      t.timestamps
    end
  end
end
