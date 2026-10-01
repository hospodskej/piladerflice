class SessionsController < ApplicationController
  def new
    return_to = params[:return_to].to_s
    session[:return_to_after_authenticating] = return_to if return_to.start_with?("/") && !return_to.start_with?("//")
  end

  def create
    user = User.find_by(email_address: params[:email_address].to_s.strip.downcase)

    if user&.authenticate(params[:password])
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
end
