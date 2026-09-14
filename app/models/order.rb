class Order < ApplicationRecord
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
