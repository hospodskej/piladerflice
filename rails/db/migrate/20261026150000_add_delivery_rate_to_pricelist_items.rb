# The "Doprava" row of the kontakt ceník shows the delivery price the shop
# charges (Admin > Doprava) instead of a price typed into the ceník.
class AddDeliveryRateToPricelistItems < ActiveRecord::Migration[8.1]
  def up
    add_column :pricelist_items, :delivery_rate, :boolean, null: false, default: false
    execute "UPDATE pricelist_items SET delivery_rate = 1 WHERE category = 'sluzby' AND item_name = 'Doprava'"
  end

  def down
    remove_column :pricelist_items, :delivery_rate
  end
end
