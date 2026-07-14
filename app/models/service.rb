class Service < ApplicationRecord
  serialize :images, type: Array, coder: JSON
end
