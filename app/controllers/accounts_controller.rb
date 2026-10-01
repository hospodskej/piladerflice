class AccountsController < ApplicationController
  before_action :require_login

  def show
  end

  private

  def require_login
    return if current_user

    session[:return_to_after_authenticating] = request.url
    redirect_to login_path, alert: "Pro zobrazení účtu se musíte přihlásit."
  end
end
