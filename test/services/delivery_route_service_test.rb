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
end
