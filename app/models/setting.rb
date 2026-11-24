# Shop-wide values the admin can change (Admin > Doprava), one row per key.
# A key without a row falls back to its default.
class Setting < ApplicationRecord
  # delivery_eur_per_km: empty means converted from the Kč price at the day's rate.
  DEFAULTS = { "delivery_czk_per_km" => "40", "delivery_eur_per_km" => "" }.freeze

  validates :key, presence: true, uniqueness: true, inclusion: { in: DEFAULTS.keys }
  validates :value, numericality: { only_integer: true, greater_than: 0, less_than_or_equal_to: 1000 },
                    if: -> { key == "delivery_czk_per_km" }
  validates :value, numericality: { greater_than: 0, less_than_or_equal_to: 100 },
                    if: -> { key == "delivery_eur_per_km" && value.present? }

  def self.fetch(key)
    find_by(key: key)&.value || DEFAULTS.fetch(key)
  end

  # Delivery to an address: Kč (incl. VAT) per kilometre, there and back.
  def self.delivery_czk_per_km
    fetch("delivery_czk_per_km").to_i
  end

  # Delivery to an address: € (incl. VAT) per kilometre, or nil to convert.
  def self.delivery_eur_per_km
    value = fetch("delivery_eur_per_km").to_s.tr(",", ".").strip
    value.empty? ? nil : BigDecimal(value)
  end

  # Both prices per km, as Money.
  def self.delivery_per_km
    Money.from(czk: delivery_czk_per_km, eur: delivery_eur_per_km)
  end
end
