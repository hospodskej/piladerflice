require "test_helper"

class ShareTagsTest < ActionDispatch::IntegrationTest
  def og(path)
    get path
    assert_response :success
    doc = Nokogiri::HTML(response.body)
    tags = doc.css("meta[property^='og:']").group_by { |m| m["property"] }.transform_values { |ms| ms.map { |m| m["content"] } }
    [tags, doc]
  end

  def with_share_images(images)
    original = Rails.application.config.x.share_images
    Rails.application.config.x.share_images = images
    yield
  ensure
    Rails.application.config.x.share_images = original
  end

  test "a shared link carries the page's title, description and address" do
    tags, doc = og("/sluzby")

    assert_equal ["website"], tags["og:type"]
    assert_equal ["Pila Derflice"], tags["og:site_name"]
    assert_equal ["Služby | Pila Derflice"], tags["og:title"]
    assert_equal [doc.at_css("meta[name=description]")["content"]], tags["og:description"]
    assert_equal ["http://www.example.com/sluzby"], tags["og:url"]
  end

  test "the German page announces its own language and address" do
    tags, = og("/at/sluzby")

    assert_equal ["Dienstleistungen | Pila Derflice"], tags["og:title"]
    assert_equal ["http://www.example.com/at/sluzby"], tags["og:url"]
    assert_equal ["de_AT"], tags["og:locale"]
    assert_equal ["cs_CZ"], tags["og:locale:alternate"]

    tags, = og("/sluzby")
    assert_equal ["cs_CZ"], tags["og:locale"]
    assert_equal ["de_AT"], tags["og:locale:alternate"]
  end

  test "e-shop category pages keep their category in the shared address" do
    tags, = og("/eshop?category=rezivo&sort=nejprodavanejsi&wood=tvrde")

    assert_equal ["http://www.example.com/eshop?category=rezivo"], tags["og:url"]
  end

  test "pages without a description (cart) still have a title and address, but no description" do
    tags, = og("/kosik")

    assert_equal ["Košík | Pila Derflice"], tags["og:title"]
    assert_equal ["http://www.example.com/kosik"], tags["og:url"]
    assert_nil tags["og:description"]
  end

  test "without a share card there is no image and the Twitter card is the small one" do
    with_share_images({}) do
      tags, doc = og("/")

      assert_nil tags["og:image"]
      assert_equal "summary", doc.at_css("meta[name='twitter:card']")["content"]
    end
  end

  test "each language gets its own share card, as an absolute address" do
    with_share_images("cs" => "logo-wm.webp", "de" => "logo-dm.webp") do
      cs, cs_doc = og("/")
      de, de_doc = og("/at")

      assert_match %r{\Ahttp://www\.example\.com/assets/logo-wm-\w+\.webp\z}, cs["og:image"].first
      assert_match %r{\Ahttp://www\.example\.com/assets/logo-dm-\w+\.webp\z}, de["og:image"].first
      assert_equal %w[1200], cs["og:image:width"]
      assert_equal %w[630], cs["og:image:height"]
      assert_equal ["Pila Derflice"], cs["og:image:alt"]
      assert_equal "summary_large_image", cs_doc.at_css("meta[name='twitter:card']")["content"]
      assert_equal "summary_large_image", de_doc.at_css("meta[name='twitter:card']")["content"]
    end
  end

  test "a language without its own card doesn't borrow the other one" do
    with_share_images("cs" => "logo-wm.webp") do
      assert_nil og("/at")[0]["og:image"]
      assert og("/")[0]["og:image"]
    end
  end

  test "quotes and tags in a product description can't break out of the tag" do
    product = catalog_products(:smrk)
    product.update!(description: %(x"><script>alert(1)</script><meta property="og:evil))

    tags, doc = og("/eshop/#{product.key}")

    assert_equal 1, tags["og:description"].size
    assert_nil tags["og:evil"]
    assert_not_includes doc.at_css("head").to_html, "<script>alert(1)</script>"
  end

  test "the card the designer delivers is picked up by file name" do
    # The real initializer scans app/assets/images/share/ for share-cs.* / share-de.*
    found = Rails.application.config.x.share_images
    assert_kind_of Hash, found
    assert found.keys.all? { |locale| %w[cs de].include?(locale) }
    assert found.values.all? { |path| path.match?(%r{\Ashare/share-(cs|de)\.(png|jpe?g)\z}) }
  end
end
