module HomeHelper
  CATEGORY_BREADCRUMB_KEYS = {
    "palivove" => "breadcrumbs.palivove_drevo",
    "rezivo" => "breadcrumbs.stavebni_rezivo",
    "zbytky" => "breadcrumbs.vyrobni_zbytky",
    "kamenivo" => "breadcrumbs.okrasne_kamenivo"
  }.freeze

  def product_breadcrumb_items(product)
    title = product.title_i18n
    category = t(CATEGORY_BREADCRUMB_KEYS.fetch(product.category))

    items = [[t("breadcrumbs.eshop"), "/eshop"]]
    items << [category, "/eshop?category=#{product.category}"] unless category == title
    items << [title, "#"]
  end
end
