-- Delivery estimates were looked up without preferring the place nearest
-- the sawmill (a "Hnanice" 353 km away). Forget them; each is looked up
-- again on its next use. Routes to full addresses stay cached.
DELETE FROM route_cache WHERE query LIKE '["chemotaxis","estimate",%';
