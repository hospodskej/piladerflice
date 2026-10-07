require "test_helper"

class LocalBusinessSchemaTest < ActionDispatch::IntegrationTest
  def schema_for(path, locale: "cs")
    get LocalizedPath.call(path, locale)
    assert_response :success
    scripts = Nokogiri::HTML(response.body).css('script[type="application/ld+json"]')
    return nil if scripts.empty?

    assert_equal 1, scripts.size
    JSON.parse(scripts.first.text)
  end

  test "the home page describes the business for search engines" do
    data = schema_for("/")

    assert_equal "https://schema.org", data["@context"]
    assert_equal "LocalBusiness", data["@type"]
    assert_equal "Pila Derflice", data["name"]
    assert_equal "Štěpán Merta", data["legalName"]
    assert_equal "http://www.example.com/", data["url"]
    assert_equal "http://www.example.com/de", schema_for("/", locale: "de")["url"]
    assert_equal "stepan.merta@seznam.cz", data["email"]
    assert_equal %w[+420602446339 +420515235527], data["telephone"]
    assert data["logo"].start_with?("http://www.example.com/assets/logo-wm"), "logo must be an absolute URL"
  end

  test "address, hours and map position match what the site shows" do
    data = schema_for("/")
    address = data["address"]

    assert_equal "PostalAddress", address["@type"]
    assert_equal ["Derflice 66", "671 28", "Znojmo", "CZ"],
                 address.values_at("streetAddress", "postalCode", "addressLocality", "addressCountry")

    hours = data["openingHoursSpecification"].first
    assert_equal %w[Monday Tuesday Wednesday Thursday Friday], hours["dayOfWeek"]
    assert_equal %w[07:00 15:30], hours.values_at("opens", "closes")

    assert_in_delta 48.81, data["geo"]["latitude"], 0.01
    assert_in_delta 16.12, data["geo"]["longitude"], 0.01

    get "/"
    assert_match "Derflice 66, 671 28 Znojmo", response.body
    assert_match "Pondělí - Pátek : 7:00 - 15:30", response.body
  end

  test "it has no self-published rating, which Google doesn't accept" do
    data = schema_for("/")

    assert_not data.key?("aggregateRating")
    assert_not data.key?("review")
  end

  test "the contact page has it too, and the text follows the language" do
    cs = schema_for("/kontakt")
    de = schema_for("/kontakt", locale: "de")

    assert_equal "LocalBusiness", cs["@type"]
    assert_equal ["Jižní Morava", "Severní Rakousko"], cs["areaServed"]
    assert_equal ["Südmähren", "Nordösterreich"], de["areaServed"]
    assert_match(/Brennholz/, de["description"])
  end

  test "pages that aren't about the business don't repeat it" do
    %w[/sluzby /sortiment /eshop /obchodni-podminky].each do |path|
      assert_nil schema_for(path), "#{path} should not carry the business markup"
    end
  end

  test "data can't end the script tag early" do
    html = ApplicationController.helpers.json_ld_tag("name" => "</script><script>alert(1)</script> & <b>")

    assert_equal 1, html.scan("</script>").size, "only the closing tag of the block itself"
    assert_not_includes html.delete_suffix("</script>"), "<script>alert"
    assert_equal "</script><script>alert(1)</script> & <b>", JSON.parse(Nokogiri::HTML.fragment(html).at_css("script").text)["name"]
  end
end
