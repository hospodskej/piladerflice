# The delivery price: the truck drives there and back at a price per
# kilometre the admin sets (Admin > Doprava, 40 Kč to start with), one trip
# whatever the amount (the price is incl. VAT, like the goods). Shown as an
# approximate price.
#
# Example from Štěpán: 50 km away -> 2 x 50 km x 40 Kč = 4 000 Kč.
class DeliveryQuote
  attr_reader :one_way_km, :czk_per_km

  # The quote for the address in the checkout, or nil (pickup, or no distance).
  def self.for_checkout(checkout)
    return nil unless checkout["shipping_method"] == "address"

    prefix = checkout.delivery_address_different? ? "delivery" : "billing"
    km = DeliveryRouteService.one_way_km(
      street: checkout["#{prefix}_street"], city: checkout["#{prefix}_city"],
      zip: checkout["#{prefix}_zip"], country: checkout["#{prefix}_country"]
    )
    km && new(one_way_km: km)
  end

  def initialize(one_way_km:, czk_per_km: Setting.delivery_czk_per_km)
    @one_way_km = one_way_km.to_i
    @czk_per_km = czk_per_km
  end

  def round_trip_km
    one_way_km * 2
  end

  def price_czk
    round_trip_km * czk_per_km
  end
end
