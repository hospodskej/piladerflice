# /llms.txt (https://llmstxt.org): a short Markdown overview of the business
# for AI assistants: what we sell, from what price, delivery, contact and
# the pages worth reading. Built from the catalogue, so it stays current.
class LlmsController < ApplicationController
  CATEGORY_NAMES = {
    "palivove" => "Firewood (palivové dřevo)",
    "rezivo" => "Construction lumber (stavební řezivo)",
    "zbytky" => "Sawmill by-products (výrobní zbytky)",
    "kamenivo" => "Decorative stone (okrasné kamenivo)"
  }.freeze

  def show
    expires_in 1.hour, public: true
    @products = CatalogProduct.active.ordered.includes(:catalog_variants).group_by(&:category)
  end

  private

  # "from 21 500 Kč for 5 PRM", "from 8 500 Kč per m³" - only where the
  # catalogue knows the unit (by-products and stone don't store one, so
  # their price is left to the product page and the price list).
  def price_note(product)
    variants = product.catalog_variants.select(&:in_stock?).presence || product.catalog_variants
    cheapest = variants.min_by(&:price_czk)
    return unless cheapest

    price = "#{helpers.number_with_delimiter(cheapest.price_czk, delimiter: ' ')} Kč"
    case product.template
    when "firewood" then "from #{price} for #{cheapest.amount_value} PRM (prostorový metr, a cubic metre of piled wood)" if cheapest.amount_value
    when "lumber" then "from #{price} per m³"
    end
  end
  helper_method :price_note
end
