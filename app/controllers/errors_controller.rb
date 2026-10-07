# Served by config.exceptions_app for unknown URLs and missing records.
class ErrorsController < ApplicationController
  MASCOTS = %w[mascot/waving.webp mascot/thumbs-up.webp mascot/hands-on-hips.webp].freeze

  # The error can come from any verb, and there's no form to protect here.
  skip_forgery_protection

  def not_found
    @mascot = MASCOTS.sample
    @breadcrumb_items = []

    respond_to do |format|
      format.html { render status: :not_found }
      format.any { head :not_found }
    end
  end
end
