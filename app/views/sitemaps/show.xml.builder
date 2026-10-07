xml.instruct!
xml.urlset(xmlns: "http://www.sitemaps.org/schemas/sitemap/0.9", "xmlns:xhtml" => "http://www.w3.org/1999/xhtml") do
  @entries.each do |entry|
    urls = { "cs" => sitemap_page_url(entry[:path], nil), "de" => sitemap_page_url(entry[:path], "de") }

    urls.each_value do |loc|
      xml.url do
        xml.loc loc
        xml.lastmod entry[:lastmod].utc.iso8601 if entry[:lastmod]
        urls.each { |locale, href| xml.tag!("xhtml:link", rel: "alternate", hreflang: locale, href: href) }
        xml.tag!("xhtml:link", rel: "alternate", hreflang: "x-default", href: urls["cs"])
      end
    end
  end
end
