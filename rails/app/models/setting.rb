# Shop-wide values the admin can change (Admin > Doprava), one row per key.
# A key without a row falls back to its default.
class Setting < ApplicationRecord
  DEFAULTS = { "delivery_czk_per_km" => "40" }.freeze

  validates :key, presence: true, uniqueness: true, inclusion: { in: DEFAULTS.keys }
  validates :value, numericality: { only_integer: true, greater_than: 0, less_than_or_equal_to: 1000 },
                    if: -> { key == "delivery_czk_per_km" }

  def self.fetch(key)
    find_by(key: key)&.value || DEFAULTS.fetch(key)
  end

  # Delivery to an address: Kč (incl. VAT) per kilometre, there and back.
  def self.delivery_czk_per_km
    fetch("delivery_czk_per_km").to_i
  end
end
