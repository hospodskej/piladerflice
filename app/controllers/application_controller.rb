class ApplicationController < ActionController::Base
  include CurrentSession

  allow_browser versions: :modern

  before_action :redirect_legacy_locale_param
  around_action :switch_locale

  private

  # Pages with personal data: the browser keeps no copy, so after signing
  # out on a shared computer, Back doesn't bring them up again.
  def no_store
    response.headers["Cache-Control"] = "no-store"
  end

  # Answer to too many attempts in a short time (see rate_limit in the
  # controllers): back to the page they came from, with a message.
  def too_many_attempts
    redirect_back fallback_location: root_path, alert: t("security.too_many_attempts"), status: :see_other
  end

  # The language is part of the URL: /at/... is (Austrian) German, everything else is
  # Czech. (It used to live in the session, so a shared link opened in Czech.)
  # Only for this request: the next one on the same server thread must not
  # start out in the previous visitor's language.
  def switch_locale(&action)
    I18n.with_locale(locale_from_path, &action)
  end

  # Every generated URL inside the /(:locale) routes gets the language of the
  # current page, so links never drop out of German (and positional arguments
  # like eshop_product_path(key) keep working). Czech is the unprefixed default.
  def default_url_options
    { locale: LocalizedPath.segment_for(I18n.locale) }
  end

  def locale_from_path
    LocalizedPath.locale_for(request.path_parameters[:locale]) || I18n.default_locale
  end

  # Links shared before the switch looked like /kontakt?locale=de. Send them
  # to the new address for good.
  def redirect_legacy_locale_param
    legacy = request.query_parameters["locale"]
    return unless legacy.present? && (request.get? || request.head?)
    return if request.path_parameters.key?(:locale) && request.path_parameters[:locale].present?
    return unless I18n.available_locales.map(&:to_s).include?(legacy)

    query = request.query_parameters.except("locale")
    target = localized_path(request.path, locale: legacy)
    target += "?#{query.to_query}" if query.any?
    redirect_to target, status: :moved_permanently
  end

  # "/kontakt#cenik" -> "/at/kontakt#cenik" in German. Paths that already start
  # with /at, or that aren't site paths, are returned as they are.
  def localized_path(path, locale: I18n.locale)
    LocalizedPath.call(path, locale)
  end
  helper_method :localized_path

  # The current page in the other language (the language switcher).
  def locale_switch_path(locale)
    path = LocalizedPath.call(LocalizedPath.strip(original_request_path), locale)
    query = request.query_parameters.except("locale")
    query.any? ? "#{path}?#{query.to_query}" : path
  end
  helper_method :locale_switch_path

  # request.path, except on error pages, where Rails rewrites it to /404.
  def original_request_path
    request.env["action_dispatch.original_path"].presence || request.path
  end

  def require_login
    return if current_user

    session[:return_to_after_authenticating] = request.url
    redirect_to login_path, alert: "Pro zobrazení účtu se musíte přihlásit."
  end

  def current_cart
    @current_cart ||= Cart.new(session)
  end
  helper_method :current_cart

  def current_checkout
    @current_checkout ||= CheckoutState.new(session)
  end
  helper_method :current_checkout

  # The place the customer gave in "Kam vozíme?", or nil.
  def current_delivery_estimate
    return @current_delivery_estimate if defined?(@current_delivery_estimate)

    @current_delivery_estimate = DeliveryEstimate.from_session(session)
  end
  helper_method :current_delivery_estimate

  # The German site delivers to Austria by default.
  def default_delivery_country
    I18n.locale == :de ? "at" : "cz"
  end
  helper_method :default_delivery_country
end
