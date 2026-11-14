class Cart
  include Rails.application.routes.url_helpers

  SESSION_KEY = :cart
  DISCOUNT_KEY = :discount_code

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
        "unit_price_czk" => catalog_variant.customer_price_czk,
        "category" => catalog_variant.catalog_product.category,
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

  # How much of the cart (after the discount, plus delivery) is VAT, at the
  # rates of the country the goods go to.
  def vat_breakdown(country:, delivery_czk: 0)
    VatBreakdown.new(
      lines: items.map { |item| [item.category, item.line_total_czk] },
      country: country, discount_czk: discount_czk, delivery_czk: delivery_czk
    )
  end

  # TODO: Minimum order with a progress bar in the cart ("Zbývá 701,20 Kč do
  # minimálního nákupu"). Štěpán wants it but says it still needs thinking
  # through. Open questions before building it:
  # - Is the minimum in Kč or in PRM / m³?
  # - Does it depend on distance (bands by km from Derflice)? If so, the bar
  #   can only show once the customer has typed a place in "Kam vozíme?".
  # - Does personal pickup in Derflice have a minimum?
  # - Under the minimum: block "Pokračovat", or just warn?
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
