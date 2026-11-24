require "net/http"
require "json"

class ExchangeRateService
  RATE_ENDPOINT = "https://api.frankfurter.dev/v2/rate/EUR/CZK"
  CACHE_KEY = "exchange_rate:eur_czk"
  CACHE_TTL = 6.hours
  REQUEST_TIMEOUT = 3

  FALLBACK_RATE = 25.0

  # "2 500 Kč" or "2500 CZK" (the German texts sometimes spell it CZK)
  AMOUNT_PATTERN = /(\d[\d\s]*)\s*(?:Kč|CZK)/

  class << self
    def czk_per_eur
      Rails.cache.fetch(CACHE_KEY, expires_in: CACHE_TTL) { fetch_live_rate }.to_f
    end

    def convert_price_string(price)
      return price if price.blank?

      rate = czk_per_eur
      price.gsub(AMOUNT_PATTERN) do
        czk_amount = ::Regexp.last_match(1).delete(" ").to_f
        "≈ #{format_eur(czk_amount / rate)}"
      end
    end

    # An amount for the page: Kč on the Czech site, € on the German one.
    # Takes a Money (its € amount, "≈" unless typed in the admin) or a plain
    # number of Kč (converted, always "≈").
    def display_amount(amount, locale: I18n.locale)
      money = amount.is_a?(Money) ? amount : Money.from(czk: amount)
      if locale.to_sym == :de
        "#{"≈ " unless money.exact_eur?}#{format_eur(money.eur)}"
      else
        format_czk(money.czk)
      end
    end

    def format_czk(amount)
      "#{ActiveSupport::NumberHelper.number_to_delimited(amount.to_i, delimiter: " ")} Kč"
    end

    def format_eur(amount)
      ActiveSupport::NumberHelper.number_to_currency(
        amount, unit: "€", format: "%n %u", precision: 2, delimiter: ".", separator: ","
      )
    end

    private

    def fetch_live_rate
      uri = URI(RATE_ENDPOINT)
      http = Net::HTTP.new(uri.host, uri.port)
      http.use_ssl = true
      http.open_timeout = REQUEST_TIMEOUT
      http.read_timeout = REQUEST_TIMEOUT

      response = http.get(uri.request_uri)
      raise "unexpected response #{response.code}" unless response.is_a?(Net::HTTPSuccess)

      rate = JSON.parse(response.body)["rate"]
      rate.presence || raise("response missing \"rate\" field")
    rescue StandardError => e
      Rails.logger.warn("[ExchangeRateService] live rate fetch failed (#{e.class}: #{e.message}); using fallback #{FALLBACK_RATE}")
      FALLBACK_RATE
    end
  end
end
