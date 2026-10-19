require_relative "boot"

require "rails"
# Pick the frameworks you want:
require "active_model/railtie"
require "active_job/railtie"
require "active_record/railtie"
require "active_storage/engine"
require "action_controller/railtie"
require "action_mailer/railtie"
# require "action_mailbox/engine"
# require "action_text/engine"
require "action_view/railtie"
# require "action_cable/engine"
require "rails/test_unit/railtie"

# Require the gems listed in Gemfile, including any gems
# you've limited to :test, :development, or :production.
Bundler.require(*Rails.groups)

module PilaDerflice
  class Application < Rails::Application
    # Initialize configuration defaults for originally generated Rails version.
    config.load_defaults 8.1

    # Please, add to the `ignore` list any other `lib` subdirectories that do
    # not contain `.rb` files, or that should not be reloaded or eager loaded.
    # Common ones are `templates`, `generators`, or `middleware`, for example.
    config.autoload_lib(ignore: %w[assets tasks])

    # Render errors (404 for unknown URLs and missing records) through our own
    # routes so they get the site layout instead of the static public/*.html.
    config.exceptions_app = routes

    # Active Storage only stores files here: no resized variants, and no
    # image analysis (nothing reads a picture's width or height). Its
    # analyzer would otherwise load libvips, if installed, at every start.
    config.active_storage.variant_processor = :disabled
    config.active_storage.analyzers = []

    # Product and promo photos are served by the app with "cache for a year":
    # a stored file never changes (a new photo gets a new address), so the
    # browser downloads each one once. The default redirect was cacheable
    # for only 5 minutes and cost two requests per photo.
    config.active_storage.resolve_model_to_route = :rails_storage_proxy

    # Pictures load when they are about to scroll into view, so a page shows
    # without first downloading every photo further down. The ones on the
    # first screen say loading: "eager" (and the main one fetchpriority:
    # "high") where they are drawn.
    config.action_view.image_loading = "lazy"
    config.action_view.image_decoding = "async"

    # Configuration for the application, engines, and railties goes here.
    #
    # These settings can be overridden in specific environments using the files
    # in config/environments, which are processed later.
    #
    # Times are shown in Czech time (stored in UTC as before): order and
    # inquiry dates, the admin's monthly counts, the weekly promo.
    config.time_zone = "Prague"
    # config.eager_load_paths << Rails.root.join("extras")

    # Don't generate system test files.
    config.generators.system_tests = nil

    config.i18n.load_path += Dir[Rails.root.join("config", "locales", "**", "*.yml")]
    config.i18n.available_locales = [:cs, :de]
    config.i18n.default_locale = :cs
  end
end
