# A guest shopping cart backed by the Rails session (no login system exists
# in this app, so there's no user to attach a persistent cart to). Wraps
# session[:cart], a Hash of line_id => line item attributes.
#
# A cart line is uniquely identified by *what* is being bought (product +
# variant + length, etc.) via #line_id, so adding the same configuration
# twice increments its quantity instead of creating a duplicate row - this
# matches the confirmation message shown in the UI ("item was added, or its
# quantity was adjusted").
#
# Line items store only language-neutral data (canonical keys, raw numbers,
# translation keys for their description "specs") so the cart displays
# correctly regardless of which language it was added in, or is later
# viewed in - see CartLineItem for how specs get turned into display text.
class Cart
  SESSION_KEY = :cart

  # Every price captured into the cart comes from data already marked
  # "vč. DPH" (VAT included) on the site's price sheets, so the "bez DPH"
  # (excl. VAT) figure shown in the cart dropdown is derived by dividing
  # back out the VAT rate - confirmed at 21% (standard Czech rate) by the
  # business owner. Adjust here if this ever differs per product category.
  VAT_RATE = 0.21

  def initialize(session)
    @session = session
    @session[SESSION_KEY] ||= {}
  end

  # Adds one unit of the given catalog variant to the cart, or increments
  # the quantity if it's already present. Price, image, and specs are all
  # read from the variant record itself - never trusted from the request -
  # so nothing about what ends up in the cart can be influenced by
  # tampering with submitted form data.
  #
  # Returns the line_id of the (new or updated) line.
  def add(catalog_variant)
    id = catalog_variant.id.to_s
    line = @session[SESSION_KEY][id]

    if line
      line["quantity"] += 1
    else
      @session[SESSION_KEY][id] = {
        "catalog_variant_id" => catalog_variant.id,
        "product_key" => catalog_variant.catalog_product.key,
        "image" => catalog_variant.catalog_product.image,
        "unit_price_czk" => catalog_variant.price_czk,
        "specs" => catalog_variant.to_cart_specs,
        "quantity" => 1
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

  # Sets a line's quantity directly (from the quantity stepper on the cart
  # page). A quantity of zero or less removes the line entirely, same as
  # clicking its remove button.
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

  # Empties the cart - called after a successful order (see
  # CheckoutController#confirm), since the order's own items_snapshot is
  # now the durable record of what was purchased.
  def clear
    @session[SESSION_KEY] = {}
  end
end
