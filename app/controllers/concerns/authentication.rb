module Authentication
  extend ActiveSupport::Concern
  include CurrentSession

  included do
    before_action :require_authentication
  end

  class_methods do
    def allow_unauthenticated_access(**options)
      skip_before_action :require_authentication, **options
    end
  end

  private

  def require_authentication
    resume_session || request_authentication
  end

  def request_authentication
    session[:return_to_after_authenticating] = request.url
    redirect_to new_admin_session_path
  end

  def after_authentication_url
    session.delete(:return_to_after_authenticating) || admin_root_path
  end
end
