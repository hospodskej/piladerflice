# Cart contents are stored in the session (see app/models/cart.rb). Rails'
# default cookie-based session store has a hard ~4KB size limit per cookie,
# which a cart with several distinct line items could realistically
# approach or exceed, causing Rails to silently fail to persist the
# session (ActionDispatch::Cookies::CookieOverflow).
#
# Switching to :cache_store keeps only a small session id in the cookie and
# stores the actual session data server-side via Rails.cache - no new gem
# needed, since this reuses whatever cache store is already configured
# (file_store by default; see config/environments/*.rb).
Rails.application.config.session_store :cache_store, key: "_piladerflice_session"
