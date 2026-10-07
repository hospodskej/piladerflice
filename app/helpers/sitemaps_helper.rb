module SitemapsHelper
  # Absolute URL for a site path in the given language: German pages live
  # under /de, Czech (the default) has no prefix.
  def sitemap_page_url(path, locale)
    request.base_url + LocalizedPath.call(path, locale || "cs")
  end
end
