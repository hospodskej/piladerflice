module CurrentSession
  extend ActiveSupport::Concern

  included do
    before_action :resume_session
    helper_method :current_user
  end

  private

  def current_user
    Current.user
  end

  def resume_session
    Current.session ||= find_session_by_cookie
  end

  # Customers stay signed in; an admin signs in again after this long, so a
  # forgotten admin login on some computer doesn't stay open for good.
  ADMIN_SESSION_LIFETIME = 30.days

  # What a visitor had before signing in and keeps afterwards.
  KEPT_ON_SIGN_IN = %w[cart discount_code checkout delivery_estimate return_to_after_authenticating csp_nonce].freeze

  def find_session_by_cookie
    record = Session.find_by(id: cookies.signed[:session_id]) if cookies.signed[:session_id]
    return record unless record&.user&.admin? && record.created_at < ADMIN_SESSION_LIFETIME.ago

    record.destroy
    cookies.delete(:session_id)
    nil
  end

  def start_new_session_for(user)
    renew_browser_session
    user.sessions.create!(user_agent: request.user_agent, ip_address: request.remote_ip).tap do |session_record|
      Current.session = session_record
      cookies.signed.permanent[:session_id] = { value: session_record.id, httponly: true, same_site: :lax }
    end
  end

  # Signing out empties the browser session (cart included, for shared
  # computers). The CSP nonce stays: Turbo keeps the page's policy.
  def terminate_session
    Current.session&.destroy
    cookies.delete(:session_id)
    nonce = session[:csp_nonce]
    reset_session
    session[:csp_nonce] = nonce if nonce
  end

  # A new session ID at sign-in, so an ID someone planted before (session
  # fixation) is worth nothing afterwards. The cart and checkout carry over.
  def renew_browser_session
    kept = KEPT_ON_SIGN_IN.filter_map { |key| [key, session[key]] unless session[key].nil? }.to_h
    reset_session
    kept.each { |key, value| session[key] = value }
  end
end
