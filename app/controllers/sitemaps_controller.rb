class SitemapsController < ApplicationController
  skip_before_action :set_locale

  def show
    expires_in 1.hour, public: true
    @entries = static_entries + category_entries + product_entries
  end

  private

  # Every entry is { path:, lastmod: }. The view lists each one in Czech (the
  # default, no parameter) and German (?locale=de) and links the two together.
  def static_entries
    [root_path, kontakt_path, sluzby_path, sortiment_path, palivove_drevo_path, stavebni_rezivo_path,
     truhlarske_rezivo_path, okrasne_kamenivo_path, vyrobni_zbytky_path, terms_path, cookies_policy_path,
     privacy_policy_path]
      .map { |path| { path: path } }
  end

  # /eshop already is the "palivove" listing, so only the other categories
  # that actually have something to show get their own entry.
  def category_entries
    categories = CatalogProduct.active.distinct.pluck(:category) - %w[palivove]
    [{ path: eshop_path }] + CatalogProduct::CATEGORIES.intersection(categories).map { |category| { path: eshop_path(category: category) } }
  end

  def product_entries
    CatalogProduct.active.ordered.includes(:catalog_variants).map do |product|
      { path: eshop_product_path(product.key), lastmod: [product.updated_at, *product.catalog_variants.map(&:updated_at)].max }
    end
  end
end
