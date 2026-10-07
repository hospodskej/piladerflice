require "test_helper"

class MetaTagsTest < ActionDispatch::IntegrationTest
  BRAND = "Pila Derflice".freeze

  # Every public, indexable page (the same set the sitemap lists), as
  # [path, params] so each one can be requested in both languages.
  def public_paths
    static = %w[/ /kontakt /sluzby /sortiment /sortiment/palivove-drevo /sortiment/stavebni-rezivo
                /sortiment/truhlarske-rezivo /sortiment/okrasne-kamenivo /sortiment/vyrobni-zbytky
                /obchodni-podminky /zasady-cookies /zasady-ochrany-osobnich-udaju]
    shop = %w[palivove rezivo zbytky kamenivo].map { |category| "/eshop?category=#{category}" }
    products = CatalogProduct.active.map { |product| "/eshop/#{product.key}" }
    static + shop + products
  end

  def meta_for(path, locale)
    get path, params: { locale: locale }
    assert_response :success, "#{path} (#{locale})"
    doc = Nokogiri::HTML(response.body)

    [doc.at_css("title")&.text, doc.at_css("meta[name=description]")&.[]("content")]
  end

  %w[cs de].each do |locale|
    test "every public page has a unique title and a unique, sensibly sized description (#{locale})" do
      metas = public_paths.index_with { |path| meta_for(path, locale) }

      metas.each do |path, (title, description)|
        assert title.present?, "#{path} has no title"
        assert_operator title.length, :<=, 70, "#{path} title is too long: #{title}"
        assert description.present?, "#{path} has no description"
        assert_operator description.length, :>=, 40, "#{path} description is too short: #{description}"
        assert_operator description.length, :<=, 160, "#{path} description is too long (#{description.length}): #{description}"
      end

      titles = metas.transform_values(&:first)
      descriptions = metas.transform_values(&:last)
      assert_equal titles.values.uniq.size, titles.size, "duplicate titles: #{titles.group_by { |_, t| t }.select { |_, v| v.size > 1 }.keys}"
      assert_equal descriptions.values.uniq.size, descriptions.size, "duplicate descriptions"
      assert titles.values.none? { |title| title == BRAND }, "a page only has the bare brand as title"
    end
  end

  test "titles are translated" do
    cs_title, = meta_for("/sluzby", "cs")
    de_title, = meta_for("/sluzby", "de")

    assert_equal "Služby | #{BRAND}", cs_title
    assert_equal "Dienstleistungen | #{BRAND}", de_title
  end

  test "the home page leads with the brand and says what the business sells" do
    title, description = meta_for("/", "cs")

    assert title.start_with?(BRAND)
    assert_match(/palivové dřevo/i, description)
  end

  test "a product page uses the product's own name and description" do
    product = catalog_products(:smrk)
    product.update!(title: "Smrk", description: "Suché <b>smrkové</b> dřevo.")

    title, description = meta_for("/eshop/#{product.key}", "cs")

    assert_match(/\ASmrk/, title)
    assert_equal "Suché smrkové dřevo.", description
  end

  test "a product without a description falls back to a generic one" do
    product = catalog_products(:smrk)
    product.update!(description: nil, description_de: nil)

    _, description = meta_for("/eshop/#{product.key}", "cs")

    assert_includes description, product.title_i18n
  end

  test "text from the database can't break out of the description attribute" do
    product = catalog_products(:smrk)
    product.update!(description: %(x"><script>alert(1)</script><meta name="y))

    get "/eshop/#{product.key}", params: { locale: "cs" }
    head = response.body[%r{<head>.*</head>}m]

    assert_not_includes head, "<script>alert(1)</script>"
    assert_equal 1, Nokogiri::HTML(response.body).css("meta[name=description]").size
  end

  test "pages that aren't for search engines still get a title of their own" do
    get "/kosik"
    assert_select "title", "Košík | #{BRAND}"

    get "/prihlaseni"
    assert_select "title", "Přihlášení | #{BRAND}"

    get "/registrace"
    assert_select "title", "Registrace | #{BRAND}"
  end

  test "checkout and account pages have titles" do
    post cart_items_path, params: { catalog_variant_id: catalog_variants(:tramy_variant).id }
    get checkout_shipping_path
    assert_select "title", "Doprava a platba | #{BRAND}"

    User.create!(email_address: "meta@example.com", password: "supersecret", role: "customer")
    post session_path, params: { email_address: "meta@example.com", password: "supersecret" }
    get account_path
    assert_select "title", "Můj účet | #{BRAND}"
    get edit_account_path
    assert_select "title", "Upravit údaje | #{BRAND}"
  end

  test "the 404 page has its own title" do
    Rails.application.env_config["action_dispatch.show_detailed_exceptions"] = false
    get "/tahle-stranka-neexistuje"
    assert_response :not_found
    assert_select "title", "Stránka nenalezena | #{BRAND}"
  ensure
    Rails.application.env_config["action_dispatch.show_detailed_exceptions"] = true
  end
end
