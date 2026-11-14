# Suggestions while typing a town ("Kam vozíme?") or an address (checkout),
# fetched by address_autocomplete_controller.js. Goes through the server so
# the openrouteservice key stays private.
class AddressSuggestionsController < ApplicationController
  # Typing sends a request every few letters; this only stops a script.
  rate_limit to: 120, within: 1.minute, with: -> { render json: [], status: :too_many_requests }

  def index
    expires_in 1.day, public: true
    render json: DeliveryRouteService.suggest(text: params[:q], country: params[:country], kind: params[:kind])
  end
end
