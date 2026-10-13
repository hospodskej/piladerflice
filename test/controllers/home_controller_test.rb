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

  def firewood_product_with(variants)
    product = CatalogProduct.create!(key: "testovaci-drevo", template: "firewood", category: "palivove", title: "Testovací", active: true)
    variants.each_with_index do |(group, stacked, length, amount), index|
      product.catalog_variants.create!(key: "v#{index}", variant_group: group, stacked: stacked, length_label: length,
                                       amount_value: amount, price_czk: 1000, position: index)
    end
    product
  end

  test "firewood page: container wood is split into a loose block and a stacked block, each with its own grid" do
    product = firewood_product_with([
      ["kontejner", false, "33 cm", 14], ["kontejner", false, "25 cm", 14],
      ["kontejner", true, "100 cm", 13], ["kontejner", true, "50 cm", 13],
      ["bedny", true, "50 cm", 1]
    ])

    get eshop_product_url(product.key)

    assert_select "#section-kontejner h3.inventory-block-title", count: 2
    titles = css_select("#section-kontejner h3.inventory-block-title").map { |t| t.text.strip }
    assert_equal ["Sypané v kontejneru", "Skládané v kontejneru"], titles

    grids = css_select("#section-kontejner .matrix-grid")
    assert_equal 2, grids.size
    assert_equal ["33 cm", "25 cm"], grids[0].css(".col-header").map { |c| c.text.strip }
    assert_equal ["100 cm", "50 cm"], grids[1].css(".col-header").map { |c| c.text.strip }
    assert_equal ["14 PRM"], grids[0].css(".row-header").map { |r| r.text.strip }, "each block lists only its own amounts"
    assert_equal ["13 PRM"], grids[1].css(".row-header").map { |r| r.text.strip }
    assert_equal ["Sypané"], grids[0].css(".inventory-card .spec-value:first-child").map { |v| v.text.strip }.uniq
    assert_equal ["Skládané"], grids[1].css(".inventory-card .spec-value:first-child").map { |v| v.text.strip }.uniq
    assert_equal [2, 2], grids.map { |g| g.css(".inventory-card").size }, "no empty cells"
    assert_equal ["--cols: 2", "--cols: 2"], grids.map { |g| g["style"] }
  end

  test "firewood page: the crates stay in their own section below, untouched by the blocks" do
    product = firewood_product_with([["kontejner", false, "33 cm", 14], ["bedny", true, "50 cm", 1], ["bedny", true, "25 cm", 1]])

    get eshop_product_url(product.key)

    assert_select "#section-kontejner .inventory-card", count: 1
    assert_select "#section-bedny .inventory-card", count: 2
    assert_select "#section-bedny h2.section-title", text: "Palivové dřevo v bednách"
  end

  test "firewood page: a product with only one kind of container wood shows one block and no divider between blocks" do
    product = firewood_product_with([["kontejner", false, "33 cm", 14], ["kontejner", false, "25 cm", 14]])

    get eshop_product_url(product.key)

    assert_select "#section-kontejner h3.inventory-block-title", count: 1
    assert_select "#section-kontejner .block-separator", count: 0
  end

  test "firewood page: the container filter still shows and hides the right sections" do
    get eshop_product_url(catalog_products(:smrk).key)

    assert_select "#section-kontejner"
    assert_select "#section-bedny"
    assert_select ".filter-btn[data-target=kontejner]"
    assert_select ".mid-separator", count: 1
  end

  test "the German firewood blocks are translated" do
    product = firewood_product_with([["kontejner", false, "33 cm", 14], ["kontejner", true, "100 cm", 13]])

    get "/at/eshop/#{product.key}"

    titles = css_select("#section-kontejner h3.inventory-block-title").map { |t| t.text.strip }
    assert_equal ["Lose im Container", "Gestapelt im Container"], titles
  end
end
