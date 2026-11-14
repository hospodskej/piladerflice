class AddDeliveryToOrders < ActiveRecord::Migration[8.1]
  def change
    # Round-trip distance and the delivery price (incl. VAT) quoted at checkout;
    # nil when the distance couldn't be worked out (price agreed individually).
    add_column :orders, :delivery_distance_km, :integer
    add_column :orders, :delivery_czk, :integer
  end
end
