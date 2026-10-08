require "test_helper"

class PromoTest < ActiveSupport::TestCase
  test "title_i18n falls back to the Czech title when there is no German translation" do
    promo = promos(:one)
    I18n.with_locale(:de) { assert_equal promo.title, promo.title_i18n }
  end

  test "the German price is shown in euros, however the crown amount is written" do
    Rails.cache.write(ExchangeRateService::CACHE_KEY, 25.0, expires_in: 1.hour)
    promo = Promo.new(price: "od 8500 Kč / m³", price_de: "ab 8500 CZK / m³")

    I18n.with_locale(:de) { assert_equal "ab ≈ 340,00 € / m³", promo.price_i18n }

    promo.price_de = nil
    I18n.with_locale(:de) { assert_equal "od ≈ 340,00 € / m³", promo.price_i18n }
  end

  test "the Czech price stays in crowns" do
    promo = Promo.new(price: "od 8500 Kč / m³", price_de: "ab 8500 CZK / m³")

    I18n.with_locale(:cs) { assert_equal "od 8500 Kč / m³", promo.price_i18n }
  end

  test "a price without an amount (per-unit wording) is left alone in German" do
    promo = Promo.new(price: "od 1500 / 1 PRMS", price_de: "ab 1500 / 1 Rm")

    I18n.with_locale(:de) { assert_equal "ab 1500 / 1 Rm", promo.price_i18n }
  end
end
