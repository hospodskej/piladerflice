class User < ApplicationRecord
  ROLES = %w[admin customer].freeze

  PROFILE_ATTRIBUTES = %w[
    first_name last_name phone
    billing_street billing_city billing_zip billing_country
    company_purchase company_name company_ico company_dic
    delivery_address_different delivery_street delivery_city delivery_zip delivery_country
  ].freeze

  has_secure_password
  has_many :sessions, dependent: :destroy
  has_many :orders, dependent: :nullify

  normalizes :email_address, with: ->(email) { email.strip.downcase }

  validates :email_address, presence: true, uniqueness: true
  validates :password, length: { minimum: 8 }, allow_nil: true
  validates :role, inclusion: { in: ROLES }

  def admin?
    role == "admin"
  end
end
