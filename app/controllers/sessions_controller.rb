class SessionsController < ApplicationController
  # Password guessing: from one address, and against one account from many.
  rate_limit to: 10, within: 3.minutes, only: :create, with: -> { too_many_attempts }
  rate_limit to: 20, within: 1.hour, only: :create, name: "per-account",
             by: -> { params[:email_address].to_s.strip.downcase }, with: -> { too_many_attempts }

  def new
    return_to = params[:return_to].to_s
    session[:return_to_after_authenticating] = return_to if local_path?(return_to)
  end

  def create
    # authenticate_by takes as long for an unknown e-mail as for a wrong
    # password, so the answer time doesn't tell which e-mails have accounts.
    user = User.authenticate_by(email_address: params[:email_address].to_s, password: params[:password].to_s)

    if user
      start_new_session_for(user)
      redirect_to(session.delete(:return_to_after_authenticating) || account_path)
    else
      flash.now[:alert] = "Nesprávný e-mail nebo heslo."
      render :new, status: :unprocessable_entity
    end
  end

  def destroy
    terminate_session
    redirect_to root_path, notice: "Byli jste odhlášeni."
  end

  private

  # A path on this site only: "/muj-ucet", not "//evil.com" or "/\evil.com",
  # which browsers read as another site.
  def local_path?(path)
    path.match?(%r{\A/(?![/\\])}) && !path.match?(/[\\[:cntrl:]]/)
  end
end
