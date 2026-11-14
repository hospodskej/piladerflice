# TODO: Order status tracking. Customers see their order history but not whether
# an order was processed, dispatched or delivered, so they call to ask. Add a
# status (e.g. processing / dispatched / delivered) with an admin UI to change
# it, and show it in the account's order history.
class Order < ApplicationRecord
  belongs_to :user, optional: true

  SHIPPING_METHODS = %w[address pickup].freeze
  PAYMENT_METHODS = %w[cash bank_transfer].freeze

  COUNTRY_NAMES = {
    "cs" => { "cz" => "Česká republika", "at" => "Rakousko" },
    "de" => { "cz" => "Tschechien", "at" => "Österreich" }
  }.freeze

  validates :first_name, :last_name, :email, :phone, presence: true
  validates :billing_street, :billing_city, :billing_zip, :billing_country, presence: true
  validates :shipping_method, inclusion: { in: SHIPPING_METHODS }
  validates :payment_method, inclusion: { in: PAYMENT_METHODS }
  validates :items_snapshot, presence: true

  validates :company_name, presence: true, if: :company_purchase?
  validates :delivery_street, :delivery_city, :delivery_zip, :delivery_country,
            presence: true, if: :delivery_address_different?

  def items
    JSON.parse(items_snapshot)
  end

  def ga_purchase_items
    items.map do |item|
      product = CatalogProduct.find_by(key: item["product_key"])
      {
        item_id: item["product_key"],
        item_name: product&.title_i18n || item["product_key"],
        price: item["unit_price_czk"].to_f,
        quantity: item["quantity"]
      }
    end
  end

  # Whether a delivery price was worked out at checkout (otherwise it is
  # agreed individually).
  def delivery_quoted?
    shipping_method == "address" && delivery_czk.present?
  end

  # { rate in % => VAT in Kč } as it was when ordered; older orders (before
  # VAT by country) were all 21 %.
  def vat_by_rate
    return { 21 => vat_czk.to_i } if vat_breakdown.blank?

    vat_breakdown.to_h { |rate, amount| [rate.to_i, amount.to_i] }.sort_by { |rate, _| -rate }.to_h
  end

  def vat_country_name(locale: I18n.locale)
    country_name(vat_country.presence || "cz", locale: locale)
  end

  def discounted?
    discount_czk.to_i.positive?
  end

  def full_name
    "#{first_name} #{last_name}".strip
  end

  def billing_country_name(locale: I18n.locale)
    country_name(billing_country, locale: locale)
  end

  def delivery_country_name(locale: I18n.locale)
    country_name(delivery_country, locale: locale)
  end

  private

  def country_name(code, locale:)
    COUNTRY_NAMES.fetch(locale.to_s, COUNTRY_NAMES["cs"]).fetch(code, code)
  end
end
