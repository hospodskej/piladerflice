class PricelistItem < ApplicationRecord
  include Translatable
  translates :item_name, :details, :subcategory

  FIREWOOD_CATEGORIES = %w[palivove_volne palivove_skladane].freeze

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
