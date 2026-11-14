module HomeHelper
  CATEGORY_BREADCRUMB_KEYS = {
    "palivove" => "breadcrumbs.palivove_drevo",
    "rezivo" => "breadcrumbs.stavebni_rezivo",
    "zbytky" => "breadcrumbs.vyrobni_zbytky",
    "kamenivo" => "breadcrumbs.okrasne_kamenivo"
  }.freeze

  # Ceník values are typed in the admin and may contain formatting like
  # m<sup>3</sup>; allow only such tags, never scripts or attributes.
  def pricelist_html(text)
    sanitize(text, tags: %w[sup sub br strong em], attributes: [])
  end

  # Texts from the database with simple formatting (line breaks, bold): the
  # same short list of tags, so a stored text can never carry a script.
  alias_method :formatted_text, :pricelist_html

  def product_breadcrumb_items(product)
    title = product.title_i18n
    category = t(CATEGORY_BREADCRUMB_KEYS.fetch(product.category))

    items = [[t("breadcrumbs.eshop"), "/eshop"]]
    items << [category, "/eshop?category=#{product.category}"] unless category == title
    items << [title, "#"]
  end
end
