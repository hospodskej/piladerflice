class Promo < ApplicationRecord
  include Translatable
  has_one_attached :image
  translates :title, :feature_1, :feature_2, :feature_3, :price

  def self.weekly_pick
    promos = order(:id).to_a
    return nil if promos.empty?

    promos[Date.current.cweek % promos.size]
  end
end
