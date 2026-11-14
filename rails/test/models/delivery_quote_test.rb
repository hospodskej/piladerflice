require "test_helper"

class DeliveryQuoteTest < ActiveSupport::TestCase
  test "Štěpán's example: 50 km away is 4 000 Kč there and back" do
    quote = DeliveryQuote.new(one_way_km: 50)

    assert_equal 100, quote.round_trip_km
    assert_equal 4000, quote.price_czk
  end

  test "the price per km is the one set in the admin" do
    Setting.create!(key: "delivery_czk_per_km", value: "55")

    assert_equal 5500, DeliveryQuote.new(one_way_km: 50).price_czk
  end
end
