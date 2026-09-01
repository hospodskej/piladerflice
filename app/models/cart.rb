require "digest/sha1"

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
  # back out the standard Czech VAT rate. Adjust here if this ever differs
  # per product category.
  VAT_RATE = 0.21

  def initialize(session)
    @session = session
    @session[SESSION_KEY] ||= {}
  end

  # Adds one unit of the given product/variant to the cart, or increments
  # the quantity if that exact configuration is already present.
  #
  # product_key   - canonical product identifier, e.g. "habr", "tramy"
  # image         - asset path used for the cart thumbnail
  # unit_price_czk - price in CZK (whole crowns) for one unit, as an Integer
  # specs         - ordered array of spec hashes describing the variant,
  #                 e.g. [{ "type" => "key", "value" => "common.loose_variant" },
  #                       { "type" => "raw", "value" => "100 cm" },
  #                       { "type" => "amount", "value" => 15, "unit_key" => "common.per_m3" }]
  #
  # Returns the line_id of the (new or updated) line.
  def add(product_key:, image:, unit_price_czk:, specs:)
    id = line_id(product_key, specs)
    line = @session[SESSION_KEY][id]

    if line
      line["quantity"] += 1
    else
      @session[SESSION_KEY][id] = {
        "product_key" => product_key,
        "image" => image,
        "unit_price_czk" => unit_price_czk,
        "specs" => specs,
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

  private

  # A stable id for a given product+variant combination, so re-adding the
  # same configuration finds the existing line instead of creating a new
  # one. Specs are language-neutral already (see class comment), so this
  # is stable across locales too.
  def line_id(product_key, specs)
    Digest::SHA1.hexdigest([product_key, specs.to_json].join("|"))
  end
end
