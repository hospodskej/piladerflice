# How VAT works for one product category (Admin > DPH): whether the prices
# typed in the admin already include VAT, and the rate in each country we
# deliver to.
#
# The customer pays the same final price wherever the goods go: the admin
# price, plus Czech VAT when it was entered without VAT. What differs by
# country is how much of that price is VAT (pickup in Derflice is always
# Czech VAT, the goods are handed over here).
class VatRule < ApplicationRecord
  COUNTRIES = %w[cz at].freeze
  CATEGORY_LABELS = {
    "palivove" => "Palivové dřevo",
    "rezivo" => "Stavební řezivo",
    "zbytky" => "Výrobní zbytky (štěpka, piliny, odkory, hobliny)",
    "kamenivo" => "Okrasné kamenivo"
  }.freeze
  DEFAULT_RATES = { "cz" => 21, "at" => 20 }.freeze

  validates :category, inclusion: { in: CatalogProduct::CATEGORIES }, uniqueness: true
  validates :rate_cz, :rate_at, numericality: { only_integer: true, greater_than_or_equal_to: 0, less_than: 100 }

  # The rule for a category; a category without a row counts its prices
  # with VAT, at the standard rates.
  def self.for(category)
    Current.vat_rules ||= all.index_by(&:category)
    Current.vat_rules[category.to_s] || new(category: category.to_s, prices_include_vat: true, rate_cz: DEFAULT_RATES["cz"], rate_at: DEFAULT_RATES["at"])
  end

  def self.country(value)
    COUNTRIES.include?(value.to_s) ? value.to_s : "cz"
  end

  def label
    CATEGORY_LABELS.fetch(category, category)
  end

  # The VAT rate in % for goods delivered to that country.
  def rate_for(country)
    self.class.country(country) == "at" ? rate_at : rate_cz
  end

  # What the customer pays for an admin price: the price itself when it was
  # entered with VAT, else the price plus Czech VAT.
  def customer_price(admin_price)
    return admin_price if admin_price.nil? || prices_include_vat?

    (admin_price * (100 + rate_cz) / 100.0).round
  end
end
