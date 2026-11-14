# The site's routes live under an optional (:locale) prefix (see routes.rb).
# Declaring the key here, even as nil, keeps positional arguments such as
# eshop_product_path("smrk") bound to :key instead of :locale in every context
# (controllers, views, mailers, console, tests). Controllers fill in the real
# language per request.
Rails.application.routes.default_url_options[:locale] = nil
