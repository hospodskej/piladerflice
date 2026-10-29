# "Kam vozíme?" in the e-shop: the approximate delivery price before checkout.
class DeliveryEstimatesController < ApplicationController
  def create
    query = params[:query].to_s.squish
    country = params[:country].presence || default_delivery_country

    if query.blank?
      @delivery_estimate_error = t("delivery_estimate.blank")
    elsif !DeliveryEstimate.lookup!(session, query: query, country: country)
      @delivery_estimate_error = t("delivery_estimate.not_found", query: query)
    end

    respond
  end

  def destroy
    DeliveryEstimate.clear(session)
    respond
  end

  private

  def respond
    respond_to do |format|
      format.turbo_stream { render :update }
      format.html do
        flash[:delivery_estimate_error] = @delivery_estimate_error
        redirect_back fallback_location: eshop_path
      end
    end
  end
end
