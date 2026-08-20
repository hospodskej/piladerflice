module EshopHelper
  # Displays an eshop.products.<key>.price locale string (e.g. "od 1 260 Kč"
  # / "ab 1.260 Kč") in the current language - as-is for Czech, or
  # live-converted to a "≈"-prefixed EUR estimate for German. These catalog
  # "starting from" prices aren't from an official EUR price sheet (unlike
  # palivové dřevo on /kontakt), so they get the same dynamic conversion as
  # every other non-firewood price - see ExchangeRateService and
  # PricelistItem#price_i18n for the equivalent logic on the price list.
  def eshop_catalog_price(product_key)
    price = t("eshop.products.#{product_key}.price")
    I18n.locale == :de ? ExchangeRateService.convert_price_string(price) : price
  end
end
