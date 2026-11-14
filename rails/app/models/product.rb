class Product < ApplicationRecord
  include Translatable
  translates :title, :text
end
