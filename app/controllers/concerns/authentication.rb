# Cookie/session-backed login, scoped entirely to the admin area - see
# Admin::BaseController. Nothing in the public storefront (home, eshop,
# cart, checkout, etc.) includes this, so none of it requires being
# logged in; this only ever gates controllers under Admin::.
#
# Follows Rails' own standard authentication generator pattern, so this
# should look familiar to any Rails developer who picks up this project
# later, and is also the natural foundation to build customer-facing
# login on top of when that's wanted (a second "customer" role already
# exists on User - see app/models/user.rb).
module Authentication
  extend ActiveSupport::Concern

  included do
    before_action :require_authentication
    helper_method :current_user
  end

  class_methods do
    def allow_unauthenticated_access(**options)
      skip_before_action :require_authentication, **options
    end
  end

  private

  def current_user
    Current.user
  end

  def require_authentication
    resume_session || request_authentication
  end

  def resume_session
    Current.session ||= find_session_by_cookie
  end

  def find_session_by_cookie
    Session.find_by(id: cookies.signed[:session_id]) if cookies.signed[:session_id]
  end

  def request_authentication
    session[:return_to_after_authenticating] = request.url
    redirect_to new_admin_session_path
  end

  def after_authentication_url
    session.delete(:return_to_after_authenticating) || admin_root_path
  end

  def start_new_session_for(user)
    user.sessions.create!(user_agent: request.user_agent, ip_address: request.remote_ip).tap do |session_record|
      Current.session = session_record
      cookies.signed.permanent[:session_id] = { value: session_record.id, httponly: true, same_site: :lax }
    end
  end

  def terminate_session
    Current.session&.destroy
    cookies.delete(:session_id)
  end
end
