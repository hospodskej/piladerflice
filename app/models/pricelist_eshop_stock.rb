# Whether a row of the price list (Ceník on the Kontakt page) is available,
# taken from the e-shop's "Skladem" ticks instead of a checkbox of its own, so
# Štěpán marks stock in one place (Katalog a ceny).
#
# Firewood rows match e-shop variants by
# - wood: the row's woods ("Jasan / Dub") against the products' names,
# - kind: "volně ložené" = kontejner, "skládané" (bedny) = bedny,
# - length: "1m" = "100 cm", "50cm" = "50 cm", ...
# Lumber and by-product rows match the product of the same name ("Prkna
# (24mm)" = Prkna), any of its variants; the stone row matches the stone
# products. A row is available when any matching variant is in stock. A row
# with no match (e.g. "Ostatní tvrdé" isn't sold in the e-shop) keeps its own
# checkbox in Admin > Ceník.
class PricelistEshopStock
  SECTION_GROUPS = { "palivove_volne" => "kontejner", "palivove_skladane" => "bedny" }.freeze
  # Price-list section => e-shop category, for rows matched by product name.
  SECTION_CATEGORIES = { "stavebni" => "rezivo", "zbytky" => "zbytky", "kamenivo" => "kamenivo" }.freeze

  # Loaded once per request.
  def self.current
    Current.pricelist_eshop_stock ||= new
  end

  def initialize
    @products = CatalogProduct.active.includes(:catalog_variants).to_a
  end

  # true/false from the e-shop, or nil when the e-shop has no matching variant.
  def available?(item)
    variants = variants_for(item)
    variants.any? ? variants.any?(&:in_stock?) : nil
  end

  # The e-shop products a row follows ("Jasan, Dub"), for the admin.
  def product_titles(item)
    products_for(item).select { |product| variants_for(item, [ product ]).any? }.map(&:title)
  end

  private

  def variants_for(item, products = products_for(item))
    variants = products.flat_map(&:catalog_variants)
    group = SECTION_GROUPS[item.category]
    return variants unless group

    length = self.class.length_cm(item.item_name)
    return [] unless length

    variants.select { |variant| variant.variant_group == group && self.class.length_cm(variant.length_label) == length }
  end

  def products_for(item)
    if SECTION_GROUPS.key?(item.category)
      names = item.subcategory.to_s.split("/").map { |name| self.class.normalize(name) }
      firewood = @products.select { |product| product.category == "palivove" }
      firewood.select { |product| names.include?(self.class.normalize(product.title)) }
    elsif (category = SECTION_CATEGORIES[item.category])
      products = @products.select { |product| product.category == category }
      # The single stone row stands for all the stone; other rows for the product of their name.
      return products if item.category == "kamenivo"

      name = self.class.normalize(item.item_name)
      products.select { |product| self.class.normalize(product.title) == name }
    else
      []
    end
  end

  # "Prkna (24mm)" -> "prkna": lowercase, no accents, no note in brackets.
  def self.normalize(name)
    I18n.transliterate(name.to_s.sub(/\(.*?\)/, "").squish.downcase)
  end

  # "1m", "1 m", "100 cm", "33cm" -> centimetres; nil when there's no length.
  def self.length_cm(text)
    match = text.to_s.match(/(\d+(?:[.,]\d+)?)\s*(cm|m)\b/i)
    return nil unless match

    value = match[1].tr(",", ".").to_f
    (match[2].downcase == "m" ? value * 100 : value).round
  end
end
