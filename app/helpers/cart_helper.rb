module CartHelper
  # Renders a cart line's full description in the current locale, e.g.
  # "Habr, Sypané, 100 cm, 15 PRM" (or "Habr, Lose, 100 cm, 15 Rm" in
  # German). Specs store only language-neutral data (translation keys, raw
  # measurements, numbers - see Cart), so this always reflects whichever
  # language the cart is currently being viewed in, regardless of which
  # language the item was originally added in.
  def cart_line_description(item)
    parts = [t("eshop.products.#{item.product_key}.title")]
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

  def cart_spec_text(spec)
    case spec["type"]
    when "key"
      t(spec["value"])
    when "amount"
      "#{spec["value"]} #{t(spec["unit_key"])}"
    else
      spec["value"].to_s
    end
  end
end
