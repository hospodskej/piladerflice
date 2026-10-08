# Served by config.exceptions_app for unknown URLs and missing records.
class ErrorsController < ApplicationController
  MASCOTS = %w[mascot/waving.webp mascot/thumbs-up.webp mascot/hands-on-hips.webp].freeze

  # The error can come from any verb, and there's no form to protect here.
  skip_forgery_protection

  # The query string of the original request is irrelevant here (and
  # request.path is /404), so no legacy redirect.
  skip_before_action :redirect_legacy_locale_param

  def not_found
    @mascot = MASCOTS.sample
    @breadcrumb_items = []

    respond_to do |format|
      format.html { render status: :not_found }
      format.any { head :not_found }
    end
  end

  private

  # Rails rewrites the path to /404 for this page, so look at the URL that was
  # asked for: /at/whatever gets the German 404.
  def locale_from_path
    LocalizedPath.locale_for(original_request_path[%r{\A/([^/?#]+)}, 1]) || I18n.default_locale
  end
end
