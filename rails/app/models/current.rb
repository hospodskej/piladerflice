class Current < ActiveSupport::CurrentAttributes
  attribute :session
  # VAT rules by category, loaded once per request (VatRule.for).
  attribute :vat_rules
  delegate :user, to: :session, allow_nil: true
end
