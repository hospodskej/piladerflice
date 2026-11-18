# Štěpán: show next to each wood in the price list whether it's in stock
# ("Jasan / Dub – Dostupné", green or red). Ticked in Admin > Ceník.
class AddAvailableToPricelistItems < ActiveRecord::Migration[8.1]
  def change
    add_column :pricelist_items, :available, :boolean, default: true, null: false
  end
end
