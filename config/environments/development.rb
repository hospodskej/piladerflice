require "active_support/core_ext/integer/time"

Rails.application.configure do
  # Settings specified here will take precedence over those in config/application.rb.

  # Make code changes take effect immediately without server restart.
  config.enable_reloading = true

  # Do not eager load code on boot.
  config.eager_load = false

  # Show full error reports.
  config.consider_all_requests_local = true

  # Enable server timing.
  config.server_timing = true

  # Enable/disable Action Controller caching. By default Action Controller caching is disabled.
  # Run rails dev:cache to toggle Action Controller caching.
  if Rails.root.join("tmp/caching-dev.txt").exist?
    config.action_controller.perform_caching = true
    config.action_controller.enable_fragment_cache_logging = true
    config.public_file_server.headers = { "cache-control" => "public, max-age=#{2.days.to_i}" }
  else
    config.action_controller.perform_caching = false
  end

  # Change to :null_store to avoid any caching.
  config.cache_store = :memory_store

  # Print deprecation notices to the Rails logger.
  config.active_support.deprecation = :log

  # Raise an error on page load if there are pending migrations.
  config.active_record.migration_error = :page_load

  # Highlight code that triggered database queries in logs.
  config.active_record.verbose_query_logs = true

  # Append comments with runtime information tags to SQL queries in logs.
  config.active_record.query_log_tags_enabled = true

  # Highlight code that triggered redirect in logs.
  config.action_dispatch.verbose_redirect_logs = true

  # Suppress logger output for asset requests.
  config.assets.quiet = true

  # Raises error for missing translations.
  # config.i18n.raise_on_missing_translations = true

  # Annotate rendered view with file names.
  config.action_view.annotate_rendered_view_with_filenames = true

  # Raise error when a before_action's only/except options reference missing actions.
  config.action_controller.raise_on_missing_callback_actions = true

  # Order confirmation emails (see OrderMailer) - in development, emails are
  # just written to the Rails log rather than actually sent, so this works
  # out of the box without any SMTP credentials configured. See
  # config/environments/production.rb for real delivery setup.
  #
  # If SMTP_USERNAME is set (via a local .env file - see .env.example),
  # this switches to actually sending real emails through that account
  # instead of just logging them, so you can confirm an order email really
  # lands in your inbox while testing locally. Leave .env unset and
  # nothing changes - emails still just get logged as before.
  config.action_mailer.default_url_options = { host: "localhost", port: 3000 }

  if ENV["SMTP_USERNAME"].present?
    config.action_mailer.delivery_method = :smtp
    smtp_port = ENV.fetch("SMTP_PORT", 587).to_i
    config.action_mailer.smtp_settings = {
      address: ENV.fetch("SMTP_ADDRESS", "smtp.gmail.com"),
      port: smtp_port,
      domain: ENV.fetch("SMTP_DOMAIN", nil),
      user_name: ENV.fetch("SMTP_USERNAME", nil),
      password: ENV.fetch("SMTP_PASSWORD", nil),
      authentication: "plain",
      # Port 465 (used by Seznam.cz, among others) expects an already-encrypted
      # connection from the start; port 587 (Gmail's default) expects a plain
      # connection that gets upgraded via STARTTLS. Picking the right one
      # based on the port means SMTP_PORT alone decides this correctly,
      # without a separate setting to get wrong.
      tls: smtp_port == 465,
      enable_starttls_auto: smtp_port != 465
    }
  else
    config.action_mailer.delivery_method = :test
  end

  config.action_mailer.perform_deliveries = true
  config.action_mailer.raise_delivery_errors = true
  config.action_mailer.perform_caching = false
end
