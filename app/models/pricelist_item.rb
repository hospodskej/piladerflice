class PricelistItem < ApplicationRecord
  include Translatable
  translates :item_name, :details, :subcategory

  FIREWOOD_CATEGORIES = %w[palivove_volne palivove_skladane].freeze

  # The columns each ceník table on the kontakt page actually shows, in the
  # order the admin edits them. Kamenivo is a single price cell.
  CATEGORY_FIELDS = {
    "stavebni" => %w[item_name details price],
    "palivove_volne" => %w[subcategory item_name price],
    "palivove_skladane" => %w[subcategory item_name price],
    "zbytky" => %w[item_name price],
    "kamenivo" => %w[price],
    "sluzby" => %w[item_name price]
  }.freeze
  CATEGORIES = CATEGORY_FIELDS.keys.freeze
  SINGLE_ITEM_CATEGORIES = %w[kamenivo].freeze

  validates :category, inclusion: { in: CATEGORIES }
  validates :price, presence: true
  validates :item_name, presence: true, if: -> { CATEGORY_FIELDS.fetch(category, []).include?("item_name") }
  validates :subcategory, presence: true, if: -> { FIREWOOD_CATEGORIES.include?(category) }

  scope :ordered, -> { order(:position, :id) }

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
