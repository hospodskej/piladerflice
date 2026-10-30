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
      hide_existing_account
      render :new, status: :unprocessable_entity
    end
  end

  private

  # "This e-mail is already registered" would tell anyone who has an
  # account. The page says only that the combination can't be used; the
  # owner gets an e-mail with the way to sign in or reset the password.
  def hide_existing_account
    return unless @user.errors.of_kind?(:email_address, :taken)

    @user.errors.delete(:email_address, :taken)
    @user.errors.add(:base, t("auth.registration_combination_invalid"))
    existing = User.find_by(email_address: @user.email_address)
    PasswordsMailer.account_exists(existing, locale: I18n.locale).deliver_now if existing
  rescue StandardError => e
    Rails.logger.error("[PasswordsMailer] #{e.class}: #{e.message}")
  end

  def registration_params
    params.require(:user).permit(:email_address, :password, :password_confirmation)
  end
end
