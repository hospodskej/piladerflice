class Cart
  include Rails.application.routes.url_helpers

  SESSION_KEY = :cart
  DISCOUNT_KEY = :discount_code

  VAT_RATE = 0.21

  def initialize(session)
    @session = session
    @session[SESSION_KEY] ||= {}
  end

  def add(catalog_variant, quantity = 1)
    quantity = quantity.to_i
    quantity = 1 if quantity < 1

    id = catalog_variant.id.to_s
    line = @session[SESSION_KEY][id]

    if line
      line["quantity"] += quantity
    else
      @session[SESSION_KEY][id] = {
        "catalog_variant_id" => catalog_variant.id,
        "product_key" => catalog_variant.catalog_product.key,
        "image" => image_url_for(catalog_variant),
        "unit_price_czk" => catalog_variant.price_czk,
        "specs" => catalog_variant.to_cart_specs,
        "quantity" => quantity
      }
    end

    id
  end

  def items
    @session[SESSION_KEY].map { |id, attrs| CartLineItem.new(id, attrs) }
  end

  def find(id)
    attrs = @session[SESSION_KEY][id]
    attrs && CartLineItem.new(id, attrs)
  end

  def update_quantity(id, quantity)
    quantity = quantity.to_i

    if quantity <= 0
      remove(id)
    elsif @session[SESSION_KEY][id]
      @session[SESSION_KEY][id]["quantity"] = quantity
    end
  end

  def remove(id)
    @session[SESSION_KEY].delete(id)
  end

  def empty?
    @session[SESSION_KEY].empty?
  end

  def total_quantity
    items.sum(&:quantity)
  end

  def subtotal_czk
    items.sum(&:line_total_czk)
  end

  def subtotal_excl_vat_czk
    (subtotal_czk / (1 + VAT_RATE)).round
  end

  # Goods after the discount code, incl. VAT.
  def total_czk
    subtotal_czk - discount_czk
  end

  # Applies a code typed in the cart. Returns nil, or why it can't be used
  # (a key under cart.discount_errors); an unusable code isn't kept.
  def apply_discount(code)
    discount = DiscountCode.lookup(code)
    reason = discount ? discount.unusable_reason(subtotal_czk) : :not_found
    return reason if reason

    @session[DISCOUNT_KEY] = discount.code
    @discount = discount
    nil
  end

  def remove_discount
    @session.delete(DISCOUNT_KEY)
    @discount = nil
  end

  # The code kept in the cart, even when it has stopped applying (say the
  # goods dropped under its minimum); see discount_unusable_reason.
  def discount
    return @discount if defined?(@discount)

    @discount = DiscountCode.lookup(@session[DISCOUNT_KEY])
  end

  def discount_unusable_reason
    discount&.unusable_reason(subtotal_czk)
  end

  def discount_applied?
    discount.present? && discount_unusable_reason.nil?
  end

  def discount_czk
    discount_applied? ? discount.amount_for(subtotal_czk) : 0
  end

  def clear
    @session[SESSION_KEY] = {}
    remove_discount
  end

  private

  def image_url_for(catalog_variant)
    attachment = catalog_variant.image.attached? ? catalog_variant.image : catalog_variant.catalog_product.image
    return nil unless attachment.attached?

    rails_blob_path(attachment, only_path: true)
  end
end
