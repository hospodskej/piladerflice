class Current < ActiveSupport::CurrentAttributes
  attribute :session
  # VAT rules by category, loaded once per request (VatRule.for).
  attribute :vat_rules
  # The e-shop's firewood stock for the price list (PricelistEshopStock).
  attribute :pricelist_eshop_stock
  delegate :user, to: :session, allow_nil: true
end
