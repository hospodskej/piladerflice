# A code a customer types into the cart (Admin > Slevové kódy). It takes a
# percentage or a fixed amount off the goods, never off the delivery.
class DiscountCode < ApplicationRecord
  KINDS = %w[percent fixed].freeze

  normalizes :code, with: ->(code) { code.strip.upcase }

  validates :code, presence: true, uniqueness: { case_sensitive: false },
                   format: { with: /\A[A-Z0-9_-]+\z/, message: "smí obsahovat jen písmena bez diakritiky, číslice, - a _" },
                   length: { maximum: 40 }
  validates :kind, inclusion: { in: KINDS }
  validates :value, numericality: { only_integer: true, greater_than: 0 }
  validates :value, numericality: { less_than_or_equal_to: 100 }, if: :percent?
  validates :usage_limit, numericality: { only_integer: true, greater_than: 0 }, allow_nil: true
  validates :min_order_czk, numericality: { only_integer: true, greater_than: 0 }, allow_nil: true

  def self.lookup(code)
    find_by(code: code.to_s.strip.upcase) if code.present?
  end

  def percent?
    kind == "percent"
  end

  # Why the code can't be used on goods worth subtotal_czk (a key under
  # cart.discount_errors), or nil when it can.
  def unusable_reason(subtotal_czk)
    return :inactive unless active?
    return :expired if valid_until && valid_until < Date.current
    return :used_up if usage_limit && times_used >= usage_limit
    return :min_order if min_order_czk && subtotal_czk < min_order_czk

    nil
  end

  def amount_for(subtotal_czk)
    percent? ? (subtotal_czk * value / 100.0).round : [value, subtotal_czk].min
  end

  # Counts one use, unless the limit was reached in the meantime.
  def redeem!
    self.class.where(id: id)
      .where("usage_limit IS NULL OR times_used < usage_limit")
      .update_all("times_used = times_used + 1") == 1
  end

  def label
    percent? ? "#{value} %" : "#{ActiveSupport::NumberHelper.number_to_delimited(value, delimiter: ' ')} Kč"
  end
end
