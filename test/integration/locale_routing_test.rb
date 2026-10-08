require "test_helper"

# The language is part of the URL: /at/... is German, everything else Czech, so
# a link always opens in the language it was shared in.
class LocaleRoutingTest < ActionDispatch::IntegrationTest
  # Internal links that deliberately are not language-specific.
  NON_PAGE_LINKS = %r{\A/(assets|rails|admin|sitemap\.xml|robots\.txt|up)(/|\z)}

  def page(path)
    get path
    assert_response :success, path
    Nokogiri::HTML(response.body)
  end

  def internal_targets(doc)
    hrefs = doc.css("a[href]:not(.lang-option)").map { |a| a["href"] }
    actions = doc.css("form[action]").map { |f| f["action"] }
    (hrefs + actions).select { |url| url.start_with?("/") && !url.start_with?("//") && url !~ NON_PAGE_LINKS }
  end

  # -- which language a URL gives you ------------------------------------------

  test "plain URLs are Czech and /at URLs are German" do
    assert_equal "cs", page("/kontakt").at_css("html")["lang"]
    assert_equal "de", page("/at/kontakt").at_css("html")["lang"]
    assert_equal "cs", page("/").at_css("html")["lang"]
    assert_equal "de", page("/at").at_css("html")["lang"]
  end

  test "the language comes from the URL alone, not from earlier visits" do
    page("/at/kontakt")
    assert_equal "cs", page("/kontakt").at_css("html")["lang"], "a Czech URL opened after a German one must be Czech"

    page("/kontakt")
    assert_equal "de", page("/at/kontakt").at_css("html")["lang"]
  end

  test "sharing a German page: a visitor with no history gets German" do
    get "/at/eshop/tramy"
    assert_response :success
    assert_equal "de", Nokogiri::HTML(response.body).at_css("html")["lang"]
    assert_nil session[:locale]
  end

  test "an unknown language prefix is not a page, and /de isn't one: the German site is the Austrian one" do
    %w[/fr/kontakt /de/kontakt /de].each do |path|
      get path
      assert_response :not_found, path
    end
  end

  # -- links stay in the language --------------------------------------------

  %w[/at /at/kontakt /at/sluzby /at/sortiment /at/sortiment/palivove-drevo /at/sortiment/stavebni-rezivo
     /at/eshop /at/eshop/tramy /at/eshop?category=rezivo /at/obchodni-podminky /at/prihlaseni /at/registrace /at/kosik].each do |path|
    test "every internal link and form on #{path} stays in German" do
      targets = internal_targets(page(path))
      stray = targets.reject { |url| url == "/at" || url.start_with?("/at/", "/at?", "/at#") || url.start_with?("#") }

      assert_empty stray, "links on #{path} that fall back to Czech: #{stray.uniq.first(8)}"
      assert_operator targets.size, :>, 5
    end
  end

  test "Czech pages never link into /at, except the language switch" do
    doc = page("/kontakt")

    assert_empty internal_targets(doc).select { |url| url.start_with?("/at") }
    assert_equal ["/kontakt", "/at/kontakt"], doc.css(".lang-option").map { |a| a["href"] }
  end

  test "the language switcher points at the same page in the other language, keeping the query" do
    doc = page("/eshop?category=rezivo")
    options = doc.css(".lang-option").map { |a| a["href"] }
    assert_equal ["/eshop?category=rezivo", "/at/eshop?category=rezivo"], options

    doc = page("/at/eshop/tramy")
    assert_equal ["/eshop/tramy", "/at/eshop/tramy"], doc.css(".lang-option").map { |a| a["href"] }

    doc = page("/at")
    assert_equal ["/", "/at"], doc.css(".lang-option").map { |a| a["href"] }
  end

  test "breadcrumbs don't show the language prefix as a page" do
    doc = page("/at/sortiment/palivove-drevo")
    crumbs = doc.css(".breadcrumb-item").map { |a| [a.text.strip, a["href"]] }

    assert_equal [["Sortiment", "/at/sortiment"], ["Brennholz", "/at/sortiment/palivove-drevo"]], crumbs
  end

  test "links from the database (service buttons) follow the language" do
    doc = page("/at")
    hrefs = doc.css("a.btn-cenik").map { |a| a["href"] }

    assert hrefs.any?, "expected service buttons on the home page"
    assert hrefs.all? { |href| href.start_with?("/at/") }, hrefs.inspect
    assert page("/").css("a.btn-cenik").all? { |a| !a["href"].start_with?("/at") }
  end

  # -- flows keep the language -------------------------------------------------

  test "adding to the cart and checking out in German stays under /at" do
    post "/at/cart_items", params: { catalog_variant_id: catalog_variants(:tramy_variant).id }
    assert_response :redirect
    assert_match %r{/at(/|\z)}, response.location

    get "/at/kosik/doprava"
    assert_response :success
    patch "/at/kosik/doprava", params: { shipping_method: "pickup", payment_method: "cash" }
    assert_redirected_to "/at/kosik/udaje"
  end

  test "logging in from the German page lands on the German account page" do
    User.create!(email_address: "gast@example.com", password: "supersecret", role: "customer")

    post "/at/prihlaseni", params: { email_address: "gast@example.com", password: "supersecret" }
    assert_redirected_to "/at/muj-ucet"
  end

  test "a protected German page sends you to the German login and back" do
    get "/at/muj-ucet"
    assert_redirected_to "/at/prihlaseni"
  end

  test "the German 404 page links stay German" do
    Rails.application.env_config["action_dispatch.show_detailed_exceptions"] = false
    get "/at/neexistuje"

    assert_response :not_found
    assert_equal "de", Nokogiri::HTML(response.body).at_css("html")["lang"]
    assert_select ".not-found a.btn-primary[href=?]", "/at"
    assert_empty internal_targets(Nokogiri::HTML(response.body)).reject { |url| url.start_with?("/at") }

    get "/neexistuje"
    assert_equal "cs", Nokogiri::HTML(response.body).at_css("html")["lang"]
    assert_select ".not-found a.btn-primary[href=?]", "/"
  ensure
    Rails.application.env_config["action_dispatch.show_detailed_exceptions"] = true
  end

  test "the admin area stays Czech and unprefixed" do
    post session_path, params: { email_address: "admin@piladerflice.cz", password: "correct-horse-battery" }
    get admin_root_path

    assert_response :success
    assert_equal "cs", I18n.locale.to_s
    assert_no_match(%r{href="/at}, response.body)
  end

  test "product images are not tagged with the language" do
    product = CatalogProduct.create!(key: "mit-bild", template: "simple_variant", category: "zbytky", title: "Mit Bild", active: true)
    product.image.attach(io: StringIO.new(File.binread(image_file("png").path)), filename: "a.png", content_type: "image/png")

    html = page("/at/eshop?category=zbytky").to_html
    assert_match %r{/rails/active_storage/blobs/redirect/}, html
    assert_no_match(%r{active_storage[^"]*\?locale=}, html)
  end

  # -- old links ---------------------------------------------------------------

  test "old ?locale=de links are redirected for good to the /at URL" do
    get "/kontakt?locale=de"
    assert_redirected_to "/at/kontakt"
    assert_equal 301, response.status

    get "/eshop?category=rezivo&locale=de&sort=x"
    assert_redirected_to "/at/eshop?category=rezivo&sort=x"

    get "/?locale=de"
    assert_redirected_to "/at"
  end

  test "an old ?locale=cs link just loses the parameter" do
    get "/kontakt?locale=cs"
    assert_redirected_to "/kontakt"
  end

  test "garbage in the old parameter is ignored, not redirected" do
    get "/kontakt?locale=xx"
    assert_response :success
    get "/kontakt?locale[]=de"
    assert_response :success
  end

  # -- search engines --------------------------------------------------------

  test "public pages declare their own canonical URL and both language versions" do
    doc = page("/at/sortiment/stavebni-rezivo")

    assert_equal "http://www.example.com/at/sortiment/stavebni-rezivo", doc.at_css("link[rel=canonical]")["href"]
    alternates = doc.css("link[rel=alternate][hreflang]").to_h { |link| [link["hreflang"], link["href"]] }
    assert_equal({ "cs" => "http://www.example.com/sortiment/stavebni-rezivo",
                   "de-AT" => "http://www.example.com/at/sortiment/stavebni-rezivo",
                   "x-default" => "http://www.example.com/sortiment/stavebni-rezivo" }, alternates)

    cs = page("/sortiment/stavebni-rezivo")
    assert_equal "http://www.example.com/sortiment/stavebni-rezivo", cs.at_css("link[rel=canonical]")["href"]
  end

  test "the home page's alternates and the e-shop categories" do
    assert_equal "http://www.example.com/at", page("/").css("link[hreflang=de-AT]").first["href"]
    assert_equal "http://www.example.com/", page("/at").css("link[hreflang=cs]").first["href"]

    doc = page("/eshop?category=rezivo&wood=tvrde&sort=nejprodavanejsi")
    assert_equal "http://www.example.com/eshop?category=rezivo", doc.at_css("link[rel=canonical]")["href"]
    assert_equal "http://www.example.com/at/eshop?category=rezivo", doc.at_css("link[hreflang=de-AT]")["href"]
  end

  test "private pages have no canonical or alternate links" do
    %w[/kosik /prihlaseni /registrace].each do |path|
      doc = page(path)
      assert_nil doc.at_css("link[rel=canonical]"), path
      assert_empty doc.css("link[hreflang]"), path
    end
  end

  test "robots.txt blocks the private areas in German as well" do
    get "/robots.txt"

    %w[/at/kosik /at/muj-ucet /at/prihlaseni /at/registrace].each do |path|
      assert_match(/^Disallow: #{Regexp.escape(path)}$/, response.body)
    end
  end
end
