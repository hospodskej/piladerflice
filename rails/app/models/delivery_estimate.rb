# Where the customer said we'd deliver, before checkout ("Kam vozíme?" in the
# e-shop), kept for the visit. It stores the distance, not the price, so a
# new price per km in the admin applies straight away. Checkout recalculates
# with the full address.
class DeliveryEstimate
  SESSION_KEY = :delivery_estimate

  attr_reader :query, :place, :zip, :country, :one_way_km

  def self.from_session(session)
    attrs = session[SESSION_KEY]
    attrs.is_a?(Hash) ? new(**attrs.symbolize_keys.slice(:query, :place, :zip, :country, :one_way_km)) : nil
  end

  # Looks the place up and keeps it. Returns the estimate, or nil when the
  # place can't be found (nothing is kept then).
  def self.lookup!(session, query:, country:)
    found = DeliveryRouteService.estimate(query: query, country: country)
    return nil unless found

    attrs = { query: query.to_s.squish, place: found[:place], zip: found[:zip], country: country.to_s.downcase, one_way_km: found[:km] }
    session[SESSION_KEY] = attrs.stringify_keys
    new(**attrs)
  end

  def self.clear(session)
    session.delete(SESSION_KEY)
  end

  def initialize(query:, place:, zip:, country:, one_way_km:)
    @query = query
    @place = place
    @zip = zip
    @country = country
    @one_way_km = one_way_km.to_i
  end

  def quote
    DeliveryQuote.new(one_way_km: one_way_km)
  end

  delegate :round_trip_km, :price_czk, to: :quote
end
