class Service < ApplicationRecord
  serialize :images, type: Array, coder: JSON

  include Translatable
  translates :title, :content, :button_text
end
