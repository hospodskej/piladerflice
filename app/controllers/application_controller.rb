class ApplicationController < ActionController::Base
  include CurrentSession

  allow_browser versions: :modern

  before_action :redirect_legacy_locale_param
  before_action :set_locale

  private

  # The language is part of the URL: /de/... is German, everything else is
  # Czech. (It used to live in the session, so a shared link opened in Czech.)
  def set_locale
    I18n.locale = locale_from_path
  end

  # Every generated URL inside the /(:locale) routes gets the language of the
  # current page, so links never drop out of German (and positional arguments
  # like eshop_product_path(key) keep working). Czech is the unprefixed default.
  def default_url_options
    { locale: I18n.locale.to_s == I18n.default_locale.to_s ? nil : I18n.locale }
  end

  def locale_from_path
    requested = request.path_parameters[:locale].to_s
    I18n.available_locales.map(&:to_s).include?(requested) ? requested : I18n.default_locale
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

  # "/kontakt#cenik" -> "/de/kontakt#cenik" in German. Paths that already start
  # with /de, or that aren't site paths, are returned as they are.
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
end
