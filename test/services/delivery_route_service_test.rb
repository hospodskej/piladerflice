require "test_helper"

class DeliveryRouteServiceTest < ActiveSupport::TestCase
  def lookup(**overrides)
    DeliveryRouteService.one_way_km(street: "Hlavní 1", city: "Brno", zip: "60200", country: "cz", **overrides)
  end

  test "geocodes the address, routes from the sawmill and rounds to whole kilometres" do
    fake_route(meters: 49_620)

    assert_equal 50, lookup

    geocode_url, route_url = @fake_route_urls
    assert_includes geocode_url, "/geocode/search/structured?"
    assert_includes geocode_url, "locality=Brno"
    assert_includes geocode_url, "country=CZ"
    assert_includes route_url, "/v2/directions/driving-car?"
    assert_includes route_url, "start=#{CGI.escape('16.12111,48.80958')}"
    assert_includes route_url, "end=#{CGI.escape('16.6,49.2')}"
    assert_includes route_url, "api_key=test-key"
  end

  test "an Austrian address is looked up in Austria" do
    fake_route

    lookup(street: "Hauptplatz 1", city: "Retz", zip: "2070", country: "at")
    assert_includes @fake_route_urls.first, "country=AT"
  end

  test "the distance is cached per address" do
    fake_route

    assert_equal 50, lookup
    assert_equal 50, lookup(street: " hlavní 1 ")
    assert_equal 2, @fake_route_urls.size
  end

  test "returns nil without an API key, without an address, or outside CZ and AT" do
    fake_route
    assert_nil lookup(street: "")
    assert_nil lookup(city: nil)
    assert_nil lookup(country: "sk")

    Rails.application.config.x.ors_api_key = nil
    assert_nil lookup
    assert_empty @fake_route_urls
  end

  test "returns nil when the address cannot be found" do
    fake_route(coordinates: nil)

    assert_nil lookup
  end

  test "returns nil when the service fails, and does not cache the failure" do
    fake_route { |_url| raise "openrouteservice answered 503" }
    assert_nil lookup

    fake_route
    assert_equal 50, lookup
  end

  test "estimates the distance to a town or PSČ, looking only for places in that country" do
    fake_route(meters: 23_400, place: "Znojmo")

    assert_equal({ km: 23, place: "Znojmo", zip: "669 02" }, DeliveryRouteService.estimate(query: " Znojmo ", country: "cz"))

    geocode_url = @fake_route_urls.first
    assert_includes geocode_url, "/geocode/search?"
    assert_includes geocode_url, "text=Znojmo"
    assert_includes geocode_url, "boundary.country=CZ"
    assert_includes geocode_url, "layers=locality"
  end

  test "an estimate is cached per place, whatever the spelling of spaces and case" do
    fake_route

    DeliveryRouteService.estimate(query: "Znojmo", country: "cz")
    DeliveryRouteService.estimate(query: " znojmo", country: "cz")
    assert_equal 2, @fake_route_urls.size
  end

  test "no estimate without a place, outside CZ and AT, or when the place isn't found" do
    fake_route
    assert_nil DeliveryRouteService.estimate(query: " ", country: "cz")
    assert_nil DeliveryRouteService.estimate(query: "Bratislava", country: "sk")

    fake_route(coordinates: nil)
    assert_nil DeliveryRouteService.estimate(query: "Neexistuje", country: "cz")
  end

  def fake_autocomplete(features)
    fake_route do |url|
      @fake_route_urls << url
      { "features" => features.map { |properties| { "properties" => properties } } }
    end
  end

  test "suggests addresses near the sawmill first, with street, town and postcode" do
    fake_autocomplete([
      { "label" => "Hlavní 12, Znojmo, Czechia", "street" => "Hlavní", "housenumber" => "12", "locality" => "Znojmo", "postalcode" => "669 02" },
      { "label" => "Hlavní, Brno, Czechia", "street" => "Hlavní", "locality" => "Brno" }
    ])

    found = DeliveryRouteService.suggest(text: "Hlavní", country: "cz", kind: "address")

    assert_equal({ label: "Hlavní 12, Znojmo, Czechia", street: "Hlavní 12", city: "Znojmo", zip: "669 02", country: "cz" }, found.first)
    assert_equal "Hlavní", found.second[:street]
    url = @fake_route_urls.first
    assert_includes url, "/geocode/autocomplete?"
    assert_includes url, "layers=address%2Cstreet"
    assert_includes url, "boundary.country=CZ"
    assert_includes url, "focus.point.lat=48.80958"
  end

  test "town suggestions look for places only and are cached" do
    fake_autocomplete([{ "label" => "Brno, Czechia", "locality" => "Brno" }])

    2.times { assert_equal "Brno", DeliveryRouteService.suggest(text: "Brn", country: "cz", kind: "place").first[:city] }
    assert_includes @fake_route_urls.first, "layers=locality"
    assert_equal 1, @fake_route_urls.size
  end

  test "no suggestions for fewer than 3 letters, outside CZ and AT, or when the service fails" do
    fake_autocomplete([{ "label" => "Brno", "locality" => "Brno" }])
    assert_empty DeliveryRouteService.suggest(text: "Br", country: "cz", kind: "place")
    assert_empty DeliveryRouteService.suggest(text: "Bratislava", country: "sk", kind: "place")
    assert_empty @fake_route_urls

    fake_route { |_url| raise "openrouteservice answered 503" }
    assert_empty DeliveryRouteService.suggest(text: "Brno", country: "cz", kind: "place")
  end
end
