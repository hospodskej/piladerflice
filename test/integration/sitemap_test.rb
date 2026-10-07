require "test_helper"

class SitemapTest < ActionDispatch::IntegrationTest
  NS = { "s" => "http://www.sitemaps.org/schemas/sitemap/0.9", "x" => "http://www.w3.org/1999/xhtml" }.freeze

  def sitemap
    get "/sitemap.xml"
    Nokogiri::XML(response.body) { |config| config.strict }
  end

  def locs(doc = sitemap)
    doc.xpath("//s:url/s:loc", NS).map(&:text)
  end

  test "sitemap.xml is well-formed XML served as application/xml" do
    doc = sitemap

    assert_response :success
    assert_equal "application/xml", response.media_type
    assert_equal "urlset", doc.root.name
    assert_equal NS["s"], doc.root.namespace.href
  end

  test "lists the public pages and every active product, with absolute URLs" do
    urls = locs

    %w[/ /kontakt /sluzby /sortiment /sortiment/stavebni-rezivo /obchodni-podminky /zasady-cookies /eshop /eshop?category=rezivo
       /eshop/smrk /eshop/tramy /eshop/odkory].each do |path|
      assert_includes urls, "http://www.example.com#{path}"
    end
    assert urls.all? { |url| url.start_with?("http://www.example.com/") }
    assert_equal urls.uniq, urls
  end

  test "leaves out inactive products, private areas and categories with nothing in them" do
    urls = locs.join("\n")

    assert_no_match(/inactive_test/, urls)
    %w[/admin /kosik /muj-ucet /prihlaseni /registrace].each { |path| assert_no_match(/#{Regexp.escape(path)}/, urls) }
    assert_no_match(/category=kamenivo/, urls)
  end

  test "reflects the catalog: a new product appears and a deactivated one disappears" do
    CatalogProduct.create!(key: "nove-latky", template: "lumber", category: "rezivo", title: "Nové")
    catalog_products(:tramy).update!(active: false)

    urls = locs
    assert_includes urls, "http://www.example.com/eshop/nove-latky"
    assert_not_includes urls, "http://www.example.com/eshop/tramy"
  end

  test "every page is listed in Czech and German and linked to its translation" do
    doc = sitemap
    urls = locs(doc)
    cs, de = "http://www.example.com/eshop/tramy", "http://www.example.com/eshop/tramy?locale=de"

    assert_includes urls, cs
    assert_includes urls, de
    [cs, de].each do |loc|
      entry = doc.at_xpath("//s:url[s:loc='#{loc}']", NS)
      alternates = entry.xpath("x:link[@rel='alternate']", NS).to_h { |link| [link["hreflang"], link["href"]] }
      assert_equal({ "cs" => cs, "de" => de, "x-default" => cs }, alternates)
    end
  end

  test "category URLs keep their own parameter when the German parameter is added" do
    assert_includes locs, "http://www.example.com/eshop?category=rezivo&locale=de"
  end

  test "lastmod is the newest of the product and its variants, in W3C format" do
    variant = catalog_variants(:tramy_variant)
    variant.update_columns(updated_at: Time.utc(2030, 1, 2, 3, 4, 5))

    entry = sitemap.at_xpath("//s:url[s:loc='http://www.example.com/eshop/tramy']", NS)
    assert_equal "2030-01-02T03:04:05Z", entry.at_xpath("s:lastmod", NS).text
  end

  test "no URL in the sitemap is blocked by robots.txt" do
    get "/robots.txt"
    disallowed = response.body.scan(/^Disallow: (\S+)$/).flatten

    locs.each do |url|
      path = URI.parse(url).request_uri
      assert disallowed.none? { |rule| path.start_with?(rule) }, "#{path} is in the sitemap but blocked by robots.txt"
    end
  end

  test "doesn't start a session or set cookies, so it stays cacheable" do
    get "/sitemap.xml"
    assert_nil response.headers["Set-Cookie"]
    assert_match(/public/, response.headers["Cache-Control"])
  end
end
