class FaqItem < ApplicationRecord
  include Translatable
  translates :title, :content
end
