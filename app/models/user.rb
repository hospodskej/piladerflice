class User < ApplicationRecord
  has_secure_password
  has_many :sessions, dependent: :destroy

  normalizes :email_address, with: ->(email) { email.strip.downcase }

  validates :email_address, presence: true, uniqueness: true
  validates :password, length: { minimum: 8 }, allow_nil: true

  # Only "admin" exists today (see the CreateUsers migration). "customer"
  # is reserved for the planned future customer-login feature - keeping
  # this as a check-by-value method rather than hardcoding "admin?" logic
  # elsewhere means that feature can add its own role without touching
  # this method.
  def admin?
    role == "admin"
  end
end
