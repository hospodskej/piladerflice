# Route distances for the delivery price (see DeliveryRouteService) come from
# openrouteservice. The API key is read from the environment (ORS_API_KEY,
# locally from the git-ignored .env) and never lives in the repository.
# Tests never call the real service; they set a key and a fake fetcher.
Rails.application.config.x.ors_api_key = Rails.env.test? ? nil : ENV["ORS_API_KEY"].presence
