# Stands in for openrouteservice: geocodes any address to a fixed point and
# answers the route with the given distance. Records the URLs it was asked for.
module FakeRouteService
  def fake_route(meters: 50_000, coordinates: [16.6, 49.2], place: "Znojmo", &block)
    @fake_route_urls = []
    @original_route_fetcher = DeliveryRouteService.fetcher
    @original_ors_api_key = Rails.application.config.x.ors_api_key
    Rails.application.config.x.ors_api_key = "test-key"
    Rails.cache.delete_matched(/\Adelivery_(route_km|estimate)/)
    DeliveryRouteService.fetcher = block || lambda { |url|
      @fake_route_urls << url
      if url.include?("/geocode/")
        properties = { "locality" => place, "postalcode" => "669 02" }
        { "features" => coordinates ? [ { "geometry" => { "coordinates" => coordinates }, "properties" => properties } ] : [] }
      else
        { "features" => [ { "properties" => { "summary" => { "distance" => meters } } } ] }
      end
    }
  end

  def restore_route_service
    return unless defined?(@original_route_fetcher)

    DeliveryRouteService.fetcher = @original_route_fetcher
    Rails.application.config.x.ors_api_key = @original_ors_api_key
    Rails.cache.delete_matched(/\Adelivery_(route_km|estimate)/)
  end
end
