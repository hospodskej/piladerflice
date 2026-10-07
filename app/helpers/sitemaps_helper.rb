module SitemapsHelper
  # Absolute URL for a site path, in German when `locale` is "de". The site
  # picks its language from ?locale=, and Czech (the default) needs no parameter.
  def sitemap_page_url(path, locale)
    uri = URI.parse(request.base_url + path)
    params = Rack::Utils.parse_nested_query(uri.query.to_s)
    params["locale"] = locale if locale
    uri.query = params.to_query.presence
    uri.to_s
  end
end
