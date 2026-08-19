class PricelistItem < ApplicationRecord
  include Translatable
  translates :item_name, :details, :subcategory

  # Palivové dřevo (firewood) prices are quoted directly from the business's
  # official printed CZK/EUR price sheets - both currencies are fixed,
  # entered values (see db/seeds.rb) and must never be algorithmically
  # converted. Every other category doesn't have a published EUR price, so
  # its German price is calculated live from the current CZK price instead.
  FIREWOOD_CATEGORIES = %w[palivove_volne palivove_skladane].freeze

  # The German-language price to display. For firewood, this is always the
  # literal price_de value entered from the official price sheet ("---"
  # when that sheet doesn't list a price for this wood/length at all - see
  # db/seeds.rb). For every other category, price_de (when present) is used
  # as a translated *template* - e.g. translating "1 strana" to "1 Seite"
  # while leaving the Kč amount untouched - and that template (or the plain
  # Czech price, if no template was needed) is then live-converted to EUR.
  def price_i18n
    cs_price = read_attribute(:price)
    return cs_price unless I18n.locale == :de

    if FIREWOOD_CATEGORIES.include?(category)
      read_attribute(:price_de).presence || cs_price
    else
      template = read_attribute(:price_de).presence || cs_price
      ExchangeRateService.convert_price_string(template)
    end
  end
end
