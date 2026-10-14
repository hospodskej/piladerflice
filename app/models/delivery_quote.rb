# The delivery price: the truck drives there and back at 40 Kč per kilometre,
# one trip whatever the amount (the price is incl. VAT, like the goods). Shown
# as an approximate price, and spread over the ordered PRM as a surcharge.
#
# Example from Štěpán: 50 km away, 13 PRM -> 2 x 50 km x 40 Kč = 4 000 Kč,
# about 308 Kč (≈ 13 €) per PRM.
class DeliveryQuote
  CZK_PER_KM = 40

  attr_reader :one_way_km, :prm

  # The quote for the address in the checkout, or nil (pickup, or no distance).
  def self.for_checkout(checkout, cart)
    return nil unless checkout["shipping_method"] == "address"

    prefix = checkout.delivery_address_different? ? "delivery" : "billing"
    km = DeliveryRouteService.one_way_km(
      street: checkout["#{prefix}_street"], city: checkout["#{prefix}_city"],
      zip: checkout["#{prefix}_zip"], country: checkout["#{prefix}_country"]
    )
    km && new(one_way_km: km, prm: prm_in(cart))
  end

  # Amount of firewood in the cart, in PRM.
  def self.prm_in(cart)
    cart.items.sum do |item|
      amount = item.specs.find { |spec| spec["type"] == "amount" && spec["unit_key"] == "common.per_m3" }
      amount ? amount["value"].to_i * item.quantity : 0
    end
  end

  def initialize(one_way_km:, prm: 0)
    @one_way_km = one_way_km.to_i
    @prm = prm.to_i
  end

  def round_trip_km
    one_way_km * 2
  end

  def price_czk
    round_trip_km * CZK_PER_KM
  end

  # What the delivery adds to each PRM of wood, or nil without firewood.
  def price_per_prm_czk
    (price_czk.to_f / prm).round if prm.positive?
  end
end
