class RegistrationsController < ApplicationController
  rate_limit to: 5, within: 1.hour, only: :create, with: -> { too_many_attempts }

  def new
    @user = User.new
  end

  def create
    @user = User.new(registration_params.merge(role: "customer"))

    if @user.save
      start_new_session_for(@user)
      redirect_to account_path, notice: "Účet byl vytvořen."
    else
      render :new, status: :unprocessable_entity
    end
  end

  private

  def registration_params
    params.require(:user).permit(:email_address, :password, :password_confirmation)
  end
end
