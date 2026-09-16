class Cart
  include Rails.application.routes.url_helpers

  SESSION_KEY = :cart

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
        "image" => image_url_for(catalog_variant.catalog_product),
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

  def clear
    @session[SESSION_KEY] = {}
  end

  private

  def image_url_for(catalog_product)
    return nil unless catalog_product.image.attached?

    rails_blob_path(catalog_product.image, only_path: true)
  end
end
