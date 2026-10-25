require "test_helper"

class HomeControllerTest < ActionDispatch::IntegrationTest
  test "should get index" do
    get root_url
    assert_response :success
  end

  test "eshop defaults to the palivove category" do
    get eshop_url
    assert_response :success
    assert_select ".product-card", count: CatalogProduct.active.where(category: "palivove").count
  end

  test "eshop filters by category" do
    get eshop_url(category: "rezivo")
    assert_response :success
    assert_select ".product-card", count: CatalogProduct.active.where(category: "rezivo").count
  end

  test "eshop excludes inactive products" do
    get eshop_url(category: "palivove")
    assert_response :success
    assert_select "h4.product-title", text: catalog_products(:inactive_product).title, count: 0
  end

  test "eshop filters firewood by wood hardness" do
    get eshop_url(category: "palivove", wood: "mekke")
    assert_response :success
    assert_select "h4.product-title", text: catalog_products(:smrk).title
  end

  test "eshop lists products in the catalogue order set in the admin" do
    smrk = catalog_products(:smrk)
    dub = smrk.dup.tap { |p| p.update!(key: "dub", title: "Dub test", position: 5) }
    smrk.catalog_variants.first.dup.tap { |v| v.update!(catalog_product: dub, key: "dub-1") }

    get eshop_url(category: "palivove")
    assert_equal [smrk.title, "Dub test"], css_select("h4.product-title").map { |el| el.text.strip }

    dub.update!(position: -1)
    get eshop_url(category: "palivove")
    assert_equal ["Dub test", smrk.title], css_select("h4.product-title").map { |el| el.text.strip }
  end

  test "the home bestsellers follow what was ordered once there are enough orders" do
    get root_url
    # The hand-picked akat, tramy, dub (only tramy is in the fixtures), then the rest.
    assert_includes css_select(".nejprodavanejsi-produkty-card").first.text, catalog_products(:tramy).title

    template = orders(:this_month)
    Order.delete_all
    5.times do
      template.dup.tap { |o| o.update!(items_snapshot: [{ product_key: "odkory", quantity: 1 }].to_json) }
    end
    get root_url
    titles = css_select(".nejprodavanejsi-produkty-card").map(&:text)
    assert_equal 3, titles.size
    assert_includes titles.first, catalog_products(:odkory).title
  end

  test "product pages render every display template" do
    %i[smrk tramy odkory].each do |name|
      product = catalog_products(name)
      get eshop_product_url(product.key)
      assert_response :success
      assert_select "h1", text: product.title
    end
  end

  test "product page breadcrumb links to the product's e-shop category" do
    get eshop_product_url(catalog_products(:tramy).key)
    assert_select "a.breadcrumb-item[href='/eshop?category=rezivo']", text: "Stavební řezivo"
  end

  test "icon-only add-to-cart button keeps an accessible label" do
    get eshop_product_url(catalog_products(:tramy).key)
    assert_select "form.add-to-cart-form button[type=submit][aria-label=?][title=?] svg", "Do košíku", "Do košíku"
  end

  test "product created in the admin gets a detail page without code changes" do
    product = CatalogProduct.create!(key: "prkna-iii", template: "lumber", category: "rezivo", title: "Prkna III")
    product.catalog_variants.create!(key: "iii-3000", grade: "III. jakostní třída", length_label: "3 000 mm", price_czk: 4_000)

    get eshop_product_url(product.key)
    assert_response :success
    assert_select "h1", text: "Prkna III"
    assert_select ".spec-value", text: "III. jakostní třída"
  end

  test "inactive and unknown products are not found" do
    get eshop_product_url(catalog_products(:inactive_product).key)
    assert_response :not_found

    get eshop_product_url("neexistuje")
    assert_response :not_found
  end

  test "kontakt ceník follows the admin order and only keeps formatting tags" do
    pricelist_items(:tramy).update!(position: -1, price: "9 800 Kč / m<sup>3</sup><script>alert(1)</script>")

    get kontakt_url
    assert_select "#cenik .cenik-group:first-of-type tr:first-child td:first-child", text: "Trámy"
    assert_select "#cenik sup", minimum: 1
    assert_select "#cenik script", count: 0
  end

  test "kontakt renders the kalkulace form with both category field sets" do
    get kontakt_url
    assert_response :success
    assert_select "fieldset.kalkulace-fields-section", count: InquiryFormOption::CATEGORIES.size
    assert_select "select[name='varianta[]'] option", text: inquiry_form_options(:palivove_varianta_skladane).value
    assert_select "select[name='polozka[]'] option", text: inquiry_form_options(:stavebni_polozka_tram).value
  end

  test "kontakt only offers the custom option on palivove's mnozstvi field" do
    get kontakt_url
    assert_response :success

    assert_select "select[name='varianta[]'] option[value='__custom__']", count: 0
    assert_select "select[name='druh[]'] option[value='__custom__']", count: 0
    assert_select "fieldset[data-category='palivove'] select[name='delka[]'] option[value='__custom__']", count: 0
    assert_select "select[name='mnozstvi[]'] option[value='__custom__']", count: 1

    assert_select "select[name='polozka[]'] option[value='__custom__']", count: 1
    assert_select "fieldset[data-category='stavebni'] select[name='delka[]'] option[value='__custom__']", count: 1
    assert_select "select[name='vyska[]'] option[value='__custom__']", count: 1
    assert_select "select[name='sirka[]'] option[value='__custom__']", count: 1
  end

  test "phone numbers and e-mail are clickable in header, footer and mobile sidebar" do
    get root_url
    assert_select ".contact-header a[href=?]", "tel:+420602446339"
    assert_select ".contact-header a[href=?]", "mailto:stepan.merta@seznam.cz"
    assert_select ".sidebar-contact a[href=?]", "tel:+420602446339"
    assert_select ".sidebar-contact a[href=?]", "mailto:stepan.merta@seznam.cz"
    assert_select "footer a[href=?]", "tel:+420602446339"
    assert_select "footer a[href=?]", "tel:+420515235527"
    assert_select "footer a[href=?]", "mailto:stepan.merta@seznam.cz"
  end

  test "contact page phone numbers are tel links" do
    get kontakt_url
    assert_select ".kontakt-list-items a[href=?]", "tel:+420602446339"
    assert_select ".kontakt-list-items a[href=?]", "tel:+420515235527"
  end

  test "firewood page: container cards say loose or stacked by the wood, crates are listed separately" do
    product = catalog_products(:smrk)
    product.catalog_variants.create!(key: "kontejner-33cm-10prm", variant_group: "kontejner", length_label: "33 cm", amount_value: 10, price_czk: 1500)
    catalog_variants(:smrk_kontejner_100cm).update!(stacked: true)

    get eshop_product_url(product.key)
    assert_response :success

    container = css_select("#section-kontejner .inventory-card").to_h do |card|
      values = card.css(".spec-value").map { |v| v.text.strip }
      [values[1], values[0]] # length => "Sypané" / "Skládané"
    end
    assert_equal "Skládané", container["100 cm"]
    assert_equal "Sypané", container["33 cm"]

    crates = css_select("#section-bedny .inventory-card .spec-value:first-child").map { |v| v.text.strip }
    assert_equal ["Skládané"], crates.uniq
  end

  test "firewood page headings: neutral 'available variants' on top, only crates below" do
    get eshop_product_url(catalog_products(:smrk).key)

    assert_select "#section-kontejner h2.section-title", text: "Dostupné varianty"
    assert_select "#section-bedny h2.section-title", text: "Palivové dřevo v bednách"
    assert_no_match(/Volně ložené/, response.body)
  end

  test "the German firewood page uses German labels" do
    catalog_variants(:smrk_kontejner_100cm).update!(stacked: true)

    get "/at/eshop/#{catalog_products(:smrk).key}"

    assert_select "#section-kontejner h2.section-title", text: "Verfügbare Varianten"
    assert_select "#section-kontejner .spec-value", text: "Gestapelt"
    assert_select "#section-bedny h2.section-title", text: "Brennholz in Kisten"
  end

  test "firewood page: loose 14 prm and stacked 13 prm wood share the last row, labelled '13 a 14 PRM'" do
    product = CatalogProduct.create!(key: "testovaci-drevo", template: "firewood", category: "palivove", title: "Testovací", active: true)
    [["100 cm", true, 13], ["50 cm", true, 13], ["33 cm", false, 14], ["25 cm", false, 14]].each_with_index do |(length, stacked, amount), index|
      product.catalog_variants.create!(key: "v#{index}", variant_group: "kontejner", stacked: stacked, length_label: length,
                                       amount_value: amount, price_czk: 1000, position: index)
    end

    get eshop_product_url(product.key)

    assert_equal ["13 a 14 PRM"], css_select("#section-kontejner .row-header").map { |r| r.text.strip }
    cards = css_select("#section-kontejner .inventory-card").map { |c| c.css(".spec-value").map { |v| v.text.strip } }
    assert_equal [["Skládané", "100 cm", "13 PRM"], ["Skládané", "50 cm", "13 PRM"],
                  ["Sypané", "33 cm", "14 PRM"], ["Sypané", "25 cm", "14 PRM"]], cards

    get "/at/eshop/#{product.key}"
    assert_equal ["13 und 14 Rm"], css_select("#section-kontejner .row-header").map { |r| r.text.strip }
  end

  test "a product page takes the same number of queries however many variants have photos" do
    { "firewood" => ->(i) { { variant_group: "kontejner", length_label: "#{20 + i} cm", amount_value: 10 } },
      "lumber" => ->(i) { { grade: "I. jakostní třída", length_label: "#{i} 000 mm" } },
      "simple_variant" => ->(_) { {} } }.each do |template, attributes|
      product = CatalogProduct.create!(key: "fotky-#{template}", template: template, category: "zbytky", title: "Fotky", active: true)
      add_variants = ->(range) do
        range.each do |index|
          variant = product.catalog_variants.create!(key: "v#{index}", price_czk: 1000, position: index, **attributes.(index))
          variant.image.attach(io: StringIO.new(File.binread(image_file("png").path)), filename: "v.png", content_type: "image/png")
        end
      end

      add_variants.(1..2)
      few = count_queries { get eshop_product_url(product.key) }
      add_variants.(3..8)
      many = count_queries { get eshop_product_url(product.key) }

      assert_response :success
      assert_select "img.inventory-img", 8
      assert_equal few, many, "#{template}: queries grew with the number of variants"
    end
  end

  test "on phones the account link lives in the menu: log in, or my account once signed in" do
    get root_url
    assert_select ".mobile-sidebar .sidebar-bottom a.sidebar-account[href=?]", login_path, text: "Přihlásit se"
    assert_select ".site-header a.account-btn[href=?]", login_path

    post session_path, params: { email_address: "admin@piladerflice.cz", password: "correct-horse-battery" }
    get "/at"
    assert_select ".mobile-sidebar .sidebar-bottom a.sidebar-account[href=?]", "/at/muj-ucet", text: "Mein Konto"
  end

  test "the inquiry form asks for name, then contact, then address, in reading order" do
    get kontakt_url

    labels = css_select("#kalkulace-form .kalkulace-grid-2 > .form-group label").map { |l| l.text.strip }
    assert_equal ["Jméno", "Příjmení", "E-mail", "Telefon", "Ulice", "Číslo popisné", "Město", "PSČ"], labels
    assert_select "input[name=jmeno][autocomplete=given-name]"
    assert_select "input[name=psc][autocomplete=postal-code]"
  end

  test "a firewood page has a filter built from its own variants" do
    product = CatalogProduct.create!(key: "filtr-drevo", template: "firewood", category: "palivove", title: "Filtr", active: true)
    [["33 cm", false, 10], ["50 cm", true, 5]].each_with_index do |(length, stacked, amount), index|
      product.catalog_variants.create!(key: "k#{index}", variant_group: "kontejner", stacked: stacked, length_label: length,
                                       amount_value: amount, price_czk: 1000, position: index)
    end
    product.catalog_variants.create!(key: "b1", variant_group: "bedny", stacked: true, length_label: "25 cm", amount_value: 1, price_czk: 900, position: 9)

    get eshop_product_url(product.key)

    assert_select "details.variant-filter[open] summary", text: /Filtr/
    assert_equal %w[loose stacked], css_select(".variant-filter input[name=kind]").map { |i| i["value"] }
    assert_equal %w[1 5 10], css_select(".variant-filter input[name=amount]").map { |i| i["value"] }
    assert_equal ["50 cm", "33 cm", "25 cm"], css_select(".variant-filter input[name=length]").map { |i| i["value"] }
    assert_select ".inventory-card[data-kind=loose][data-amount='10'][data-length='33 cm']"
    assert_select "#section-bedny .inventory-card[data-kind=stacked][data-amount='1'][data-length='25 cm']"
  end

  test "each box in the inquiry form carries its own name for phones" do
    get kontakt_url

    assert_equal %w[Varianta Druh Délka Množství],
                 css_select("[data-category=palivove] .kalkulace-row .kalkulace-field-label").map { |l| l.text.strip }
  end

  test "pictures further down the page load lazily, the main one straight away" do
    get eshop_product_url(catalog_products(:smrk).key)

    assert_select "header img.logo[loading=eager]"
    assert_select "img.inventory-img[loading=lazy]"
    assert_select "footer img[loading=lazy]"
  end

  private

  def count_queries(&block)
    count = 0
    counter = ->(*, payload) { count += 1 unless payload[:name].in?(["SCHEMA", "TRANSACTION"]) || payload[:cached] }
    ActiveSupport::Notifications.subscribed(counter, "sql.active_record", &block)
    count
  end
end
