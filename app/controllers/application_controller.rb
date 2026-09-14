class ApplicationController < ActionController::Base
  allow_browser versions: :modern

  before_action :set_locale

  private

  def set_locale
    requested_locale = params[:locale].to_s

    if I18n.available_locales.map(&:to_s).include?(requested_locale)
      session[:locale] = requested_locale
    end

    I18n.locale = session[:locale].presence || I18n.default_locale
  end

  def locale_switch_path(locale)
    query = request.query_parameters.merge(locale: locale)
    "#{request.path}?#{query.to_query}"
  end
  helper_method :locale_switch_path

  def current_cart
    @current_cart ||= Cart.new(session)
  end
  helper_method :current_cart

  def current_checkout
    @current_checkout ||= CheckoutState.new(session)
  end
  helper_method :current_checkout
end
