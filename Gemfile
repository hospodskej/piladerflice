source "https://rubygems.org"

# Bundle edge Rails instead: gem "rails", github: "rails/rails", branch: "main"
gem "rails", "~> 8.1.3"

# Ruby 4.0+ no longer bundles fiddle by default, but reline (used by irb /
# `rails console`) needs it on Windows specifically - without this,
# `rails console` crashes immediately on startup.
gem "fiddle"
# The modern asset pipeline for Rails [https://github.com/rails/propshaft]
gem "propshaft"
# Use sqlite3 as the database for Active Record
gem "sqlite3", ">= 2.1"
# Use the Puma web server [https://github.com/puma/puma]
gem "puma", ">= 5.0"

# Windows does not include zoneinfo files, so bundle the tzinfo-data gem
gem "tzinfo-data", platforms: %i[ windows jruby ]

group :development, :test do
  # See https://guides.rubyonrails.org/debugging_rails_applications.html#debugging-with-the-debug-gem
  gem "debug", platforms: %i[ mri windows ], require: "debug/prelude"

  # Loads environment variables (SMTP credentials, etc.) from a local .env
  # file that's never committed to git (already covered by .gitignore's
  # `/.env*` rule) - see .env.example for what's needed.
  gem "dotenv-rails"
end

gem "importmap-rails", "~> 2.2"

gem "turbo-rails", "~> 2.0"
gem "stimulus-rails", "~> 1.3"

# Needed by ActionMailer's SMTP delivery method (net/smtp is no longer part
# of Ruby's default gems as of Ruby 3.1+).
gem "net-smtp", require: false

# Password hashing for admin login (has_secure_password) - see app/models/user.rb.
gem "bcrypt", "~> 3.1.7"
