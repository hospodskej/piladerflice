require "net/http"
require "json"

# Live-converts CZK prices to EUR for the German price list, for every
# price-list category *except* palivové dřevo (firewood), which has its own
# official, fixed EUR pricing entered directly (see PricelistItem and
# db/seeds.rb) and should never be algorithmically converted.
#
# Source: Frankfurter (https://frankfurter.dev), a free, keyless exchange
# rate API blending official rates from the ECB and other central banks -
# no API key or paid account needed.
class ExchangeRateService
  RATE_ENDPOINT = "https://api.frankfurter.dev/v2/rate/EUR/CZK"
  CACHE_KEY = "exchange_rate:eur_czk"
  CACHE_TTL = 6.hours
  REQUEST_TIMEOUT = 3 # seconds

  # Only used if the live API is unreachable (network hiccup, API down),
  # so the price list still renders a reasonable estimate instead of
  # crashing or showing no price at all - this is a safety net, not
  # something shown to customers under normal operation. Worth nudging
  # this occasionally to keep it a plausible ballpark; the live rate is
  # always tried first, on every cache expiry (see CACHE_TTL above).
  FALLBACK_RATE = 25.0

  # Matches a whole-number CZK amount (optionally space-grouped, e.g.
  # "9 800") immediately followed by "Kč", so it converts the number
  # without touching unrelated digits elsewhere in the string (e.g. the
  # "3" in "m<sup>3</sup>", or the "1" in "1 strana").
  AMOUNT_PATTERN = /(\d[\d\s]*)\s*Kč/

  class << self
    # How many CZK equal 1 EUR right now. Cached for CACHE_TTL so normal
    # page views never wait on an external API call.
    def czk_per_eur
      Rails.cache.fetch(CACHE_KEY, expires_in: CACHE_TTL) { fetch_live_rate }.to_f
    end

    # Replaces every "<amount> Kč" found in +price+ with its live EUR
    # equivalent (prefixed with "≈" to signal it's a converted estimate,
    # not a fixed price), leaving everything else in the string - units,
    # ranges, HTML tags, already-German words - untouched. Strings with no
    # convertible amount (e.g. "Po domluvě") are returned unchanged.
    def convert_price_string(price)
      return price if price.blank?

      rate = czk_per_eur
      price.gsub(AMOUNT_PATTERN) do
        czk_amount = ::Regexp.last_match(1).delete(" ").to_f
        "≈ #{format_eur(czk_amount / rate)}"
      end
    end

    # Formats a plain numeric CZK amount (e.g. from the cart, which stores
    # prices as integers rather than "<amount> Kč" strings) for display in
    # +locale+: as Czech crowns as-is, or live-converted to a "≈"-prefixed
    # EUR estimate for German.
    def display_amount(czk_amount, locale: I18n.locale)
      if locale.to_sym == :de
        "≈ #{format_eur(czk_amount / czk_per_eur)}"
      else
        formatted = ActiveSupport::NumberHelper.number_to_delimited(czk_amount.to_i, delimiter: " ")
        "#{formatted} Kč"
      end
    end

    private

    def format_eur(amount)
      ActiveSupport::NumberHelper.number_to_currency(
        amount, unit: "€", format: "%n %u", precision: 2, delimiter: ".", separator: ","
      )
    end

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
