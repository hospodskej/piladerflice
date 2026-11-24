# Štěpán: the Austrian site shouldn't show prices converted at the day's
# exchange rate (they'd change every day), but euro prices he sets himself.
# - catalog_variants.price_eur: the € price, typed like the Kč one (with or
#   without VAT per category, Admin > DPH); empty = converted as before.
# - discount_codes.value_eur: a fixed discount in €; empty = converted.
# - orders.currency / eur_amounts: an order placed on the Austrian site is in
#   € - its goods, discount, delivery, VAT and total as the customer saw them.
# The € price per km for delivery is a Setting (delivery_eur_per_km).
class AddEuroPrices < ActiveRecord::Migration[8.1]
  def change
    add_column :catalog_variants, :price_eur, :decimal, precision: 10, scale: 2
    add_column :discount_codes, :value_eur, :decimal, precision: 10, scale: 2
    add_column :orders, :currency, :string, default: "CZK", null: false
    add_column :orders, :eur_amounts, :json
  end
end
