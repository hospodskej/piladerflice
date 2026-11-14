class Promo < ApplicationRecord
  include Translatable
  has_one_attached :image
  include OptimizesUploadedImage
  translates :title, :feature_1, :feature_2, :feature_3, :price

  # German visitors pay in euros: amounts in crowns ("od 8500 Kč", "ab 8500 CZK")
  # are shown converted, the same way as in the price list.
  alias_method :translated_price, :price_i18n

  def price_i18n
    text = translated_price
    I18n.locale == :de ? ExchangeRateService.convert_price_string(text) : text
  end

  def self.weekly_pick
    promos = order(:id).to_a
    return nil if promos.empty?

    promos[Date.current.cweek % promos.size]
  end
end
