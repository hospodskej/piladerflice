class PricelistItem < ApplicationRecord
  include Translatable
  translates :item_name, :details, :price, :subcategory
end
