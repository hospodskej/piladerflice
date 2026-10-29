require "net/http"
require "json"

# Driving distance from the sawmill to a delivery address, via openrouteservice:
# the address is geocoded, then routed by car from Derflice. Results are cached
# per address. Any problem (no key, address not found, service down) returns
# nil, and checkout falls back to "delivery price on request".
class DeliveryRouteService
  BASE_URL = "https://api.openrouteservice.org".freeze
  ORIGIN = [16.12111, 48.80958].freeze # Pila Derflice, Derflice 66 (lon, lat)
  COUNTRIES = %w[cz at].freeze
  TIMEOUT = 4
  CACHE_TTL = 30.days
  # What the e-shop estimate looks for: towns, parts of towns and postcodes,
  # so "Znojmo" doesn't find a street or a shop of that name elsewhere.
  PLACE_LAYERS = "locality,localadmin,borough,neighbourhood,postalcode".freeze

  # Fetches a JSON document; swapped out in tests.
  class_attribute :fetcher, default: lambda { |url|
    uri = URI(url)
    http = Net::HTTP.new(uri.host, uri.port)
    http.use_ssl = true
    http.open_timeout = TIMEOUT
    http.read_timeout = TIMEOUT
    response = http.get(uri.request_uri, "Accept" => "application/json, application/geo+json")
    raise "openrouteservice answered #{response.code}" unless response.is_a?(Net::HTTPSuccess)

    JSON.parse(response.body)
  }

  class << self
    # One-way driving distance in whole kilometres, or nil.
    def one_way_km(street:, city:, zip:, country:)
      country = country.to_s.downcase
      return nil if api_key.blank? || street.blank? || city.blank? || !COUNTRIES.include?(country)

      cache_key = ["delivery_route_km", country, zip, city, street].map { |part| part.to_s.squish.downcase }.join("|")
      cached = Rails.cache.read(cache_key)
      return cached if cached

      destination = geocode(street: street, city: city, zip: zip, country: country)
      return nil unless destination

      meters = route_meters(destination)
      return nil unless meters

      (meters / 1000.0).round.tap { |km| Rails.cache.write(cache_key, km, expires_in: CACHE_TTL) }
    rescue StandardError => e
      Rails.logger.warn("[DeliveryRouteService] no distance for #{city} (#{country}): #{e.class}: #{e.message}")
      nil
    end

    # The estimate in the e-shop, before the customer gives a full address:
    # the distance to a town or PSČ. Returns { km:, place:, zip: } (place is
    # the town as found, e.g. "Znojmo"), or nil.
    def estimate(query:, country:)
      country = country.to_s.downcase
      query = query.to_s.squish
      return nil if api_key.blank? || query.blank? || query.length > 80 || !COUNTRIES.include?(country)

      cache_key = ["delivery_estimate", country, query.downcase].join("|")
      cached = Rails.cache.read(cache_key)
      return cached if cached

      place = geocode_place(query, country)
      return nil unless place

      meters = route_meters(place[:coordinates])
      return nil unless meters

      { km: (meters / 1000.0).round, place: place[:name], zip: place[:zip] }
        .tap { |found| Rails.cache.write(cache_key, found, expires_in: CACHE_TTL) }
    rescue StandardError => e
      Rails.logger.warn("[DeliveryRouteService] no estimate for #{query} (#{country}): #{e.class}: #{e.message}")
      nil
    end

    private

    def geocode_place(query, country)
      params = { :api_key => api_key, :text => query, "boundary.country" => country.upcase, :layers => PLACE_LAYERS, :size => 1 }
      feature = fetcher.call("#{BASE_URL}/geocode/search?#{params.to_query}").dig("features", 0)
      coordinates = feature&.dig("geometry", "coordinates")
      return nil unless coordinates.is_a?(Array) && coordinates.size == 2

      properties = feature["properties"] || {}
      { coordinates: coordinates, name: properties["locality"].presence || properties["name"].presence || query, zip: properties["postalcode"] }
    end

    def api_key
      Rails.application.config.x.ors_api_key
    end

    # [lon, lat] of the address, or nil.
    def geocode(street:, city:, zip:, country:)
      query = { api_key: api_key, address: street, locality: city, postalcode: zip, country: country.upcase, size: 1 }
      data = fetcher.call("#{BASE_URL}/geocode/search/structured?#{query.compact_blank.to_query}")
      coordinates = data.dig("features", 0, "geometry", "coordinates")
      coordinates if coordinates.is_a?(Array) && coordinates.size == 2
    end

    def route_meters(destination)
      query = { api_key: api_key, start: ORIGIN.join(","), end: destination.join(",") }
      data = fetcher.call("#{BASE_URL}/v2/directions/driving-car?#{query.to_query}")
      data.dig("features", 0, "properties", "summary", "distance")
    end
  end
end
