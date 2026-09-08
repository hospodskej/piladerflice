module CartHelper
  # Renders a cart line's full description in the current locale, e.g.
  # "Habr, Sypané, 100 cm, 15 PRM" (or "Habr, Lose, 100 cm, 15 Rm" in
  # German). Specs store only language-neutral data (translation keys, raw
  # measurements, numbers, or admin-entered bilingual text - see Cart and
  # CatalogVariant#to_cart_specs), so this always reflects whichever
  # language the cart is currently being viewed in, regardless of which
  # language the item was originally added in.
  def cart_line_description(item)
    parts = [catalog_product_title(item.product_key)]
    parts.concat(item.specs.map { |spec| cart_spec_text(spec) })
    parts.join(", ")
  end

  # Formats a CZK amount for display in the current locale - Kč as-is for
  # Czech, or a live "≈"-prefixed EUR estimate for German, consistent with
  # how the price list on /kontakt handles currency (see
  # ExchangeRateService and PricelistItem#price_i18n).
  def cart_price(amount_czk)
    ExchangeRateService.display_amount(amount_czk)
  end

  private

  # Prefers the admin-managed CatalogProduct record; falls back to the
  # older locale-file-driven title if no matching DB record exists yet
  # (a safety net during the transition to admin-managed products, and
  # for any product a future admin might delete while it's still
  # referenced by an old cart/order).
  def catalog_product_title(product_key)
    product = CatalogProduct.find_by(key: product_key)
    product ? product.title_i18n : t("eshop.products.#{product_key}.title")
  end

  def cart_spec_text(spec)
    case spec["type"]
    when "key"
      t(spec["value"])
    when "amount"
      "#{spec["value"]} #{t(spec["unit_key"])}"
    when "bilingual"
      I18n.locale == :de ? (spec["value_de"].presence || spec["value"]) : spec["value"]
    else
      spec["value"].to_s
    end
  end
end

