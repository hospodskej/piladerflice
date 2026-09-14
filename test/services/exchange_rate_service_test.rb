require "test_helper"

class ExchangeRateServiceTest < ActiveSupport::TestCase
  setup do
    Rails.cache.write(ExchangeRateService::CACHE_KEY, 25.0, expires_in: ExchangeRateService::CACHE_TTL)
  end

  test "convert_price_string converts each Kč amount using the cached rate" do
    assert_equal "≈ 100,00 €", ExchangeRateService.convert_price_string("2 500 Kč")
  end

  test "convert_price_string converts multiple amounts in the same string" do
    result = ExchangeRateService.convert_price_string("9 800 Kč / m<sup>3</sup>")
    assert_equal "≈ 392,00 € / m<sup>3</sup>", result
  end

  test "convert_price_string leaves strings without a Kč amount untouched" do
    assert_equal "Po domluvě", ExchangeRateService.convert_price_string("Po domluvě")
  end

  test "convert_price_string returns blank input unchanged" do
    assert_nil ExchangeRateService.convert_price_string(nil)
    assert_equal "", ExchangeRateService.convert_price_string("")
  end

  test "display_amount formats Czech amounts as Kč without conversion" do
    I18n.with_locale(:cs) do
      assert_equal "1 500 Kč", ExchangeRateService.display_amount(1500)
    end
  end

  test "display_amount converts to euros for the German locale" do
    I18n.with_locale(:de) do
      assert_equal "≈ 60,00 €", ExchangeRateService.display_amount(1500)
    end
  end

  test "czk_per_eur falls back to FALLBACK_RATE when the live API is unreachable" do
    Rails.cache.delete(ExchangeRateService::CACHE_KEY)
    original_new = Net::HTTP.method(:new)
    Net::HTTP.define_singleton_method(:new) { |*| raise SocketError, "no network in test" }

    begin
      assert_equal ExchangeRateService::FALLBACK_RATE, ExchangeRateService.czk_per_eur
    ensure
      Net::HTTP.define_singleton_method(:new, original_new)
    end
  end
end
