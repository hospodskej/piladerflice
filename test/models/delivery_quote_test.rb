require "test_helper"

class DeliveryQuoteTest < ActiveSupport::TestCase
  test "Štěpán's example: 50 km away, 13 PRM" do
    quote = DeliveryQuote.new(one_way_km: 50, prm: 13)

    assert_equal 100, quote.round_trip_km
    assert_equal 4000, quote.price_czk
    assert_equal 308, quote.price_per_prm_czk
  end

  test "one trip whatever the amount, and no per-PRM price without firewood" do
    assert_equal 4000, DeliveryQuote.new(one_way_km: 50, prm: 26).price_czk
    assert_nil DeliveryQuote.new(one_way_km: 50).price_per_prm_czk
  end

  test "prm_in counts the firewood amount times the quantity" do
    cart = Cart.new({})
    cart.add(catalog_variants(:smrk_kontejner_100cm))
    cart.add(catalog_variants(:smrk_bedny), 2)
    cart.add(catalog_variants(:tramy_variant))

    assert_equal 17, DeliveryQuote.prm_in(cart)
  end
end
