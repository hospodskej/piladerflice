class ApplicationController < ActionController::Base
  # Only allow modern browsers supporting webp images, web push, badges, import maps, CSS nesting, and CSS :has.
  allow_browser versions: :modern

  before_action :set_locale

  private

  # Determines which language to render the site in.
  #
  # If the visitor just clicked one of the flag buttons in the header, the
  # request carries a `?locale=de` (or `cs`) query param — we remember that
  # choice in the session so it "sticks" as they browse to other pages that
  # don't have the param (no need to litter every link in the app with a
  # locale, which keeps this safe to add on top of the existing hardcoded
  # paths used throughout the views).
  def set_locale
    requested_locale = params[:locale].to_s

    if I18n.available_locales.map(&:to_s).include?(requested_locale)
      session[:locale] = requested_locale
    end

    I18n.locale = session[:locale].presence || I18n.default_locale
  end

  # Builds "current page, but in the given language" — used by the header's
  # language switcher so it swaps locale without losing the page you're on
  # or any filters/query params already in the URL (e.g. the eshop filters).
  def locale_switch_path(locale)
    query = request.query_parameters.merge(locale: locale)
    "#{request.path}?#{query.to_query}"
  end
  helper_method :locale_switch_path

  # The guest shopping cart for this visitor (see app/models/cart.rb).
  # Memoized per-request since several places on a page can end up reading
  # it (header widget, "added to cart" confirmation, etc).
  def current_cart
    @current_cart ||= Cart.new(session)
  end
  helper_method :current_cart

  # The visitor's in-progress checkout wizard state (see
  # app/models/checkout_state.rb).
  def current_checkout
    @current_checkout ||= CheckoutState.new(session)
  end
  helper_method :current_checkout
end
