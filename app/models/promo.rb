class Promo < ApplicationRecord
  include Translatable
  translates :title, :feature_1, :feature_2, :feature_3, :price
end
