require "test_helper"

class PricelistItemTest < ActiveSupport::TestCase
  setup do
    Rails.cache.write(ExchangeRateService::CACHE_KEY, 25.0, expires_in: ExchangeRateService::CACHE_TTL)
  end

  test "price_i18n returns the Czech price as-is for the Czech locale" do
    item = pricelist_items(:fosny)
    I18n.with_locale(:cs) { assert_equal item.price, item.price_i18n }
  end

  test "price_i18n uses the fixed German price for firewood categories, without conversion" do
    item = pricelist_items(:palivove_jasan)
    I18n.with_locale(:de) { assert_equal "67 €", item.price_i18n }
  end

  test "price_i18n live-converts the Czech price for non-firewood categories with no German override" do
    item = pricelist_items(:kamenivo)
    I18n.with_locale(:de) { assert_equal "≈ 0,04 € / 1 kg", item.price_i18n }
  end
end
