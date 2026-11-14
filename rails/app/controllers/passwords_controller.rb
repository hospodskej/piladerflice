# "Zapomenuté heslo": a link to set a new password, sent by e-mail.
#
# The answer to a request is the same whether or not the e-mail has an
# account, so the form can't be used to find out who is registered. The
# link works for 15 minutes and only until the password is changed (Rails'
# password_reset_token). Setting a new password signs the account out
# everywhere.
class PasswordsController < ApplicationController
  rate_limit to: 5, within: 10.minutes, only: :create, with: -> { too_many_attempts }
  rate_limit to: 10, within: 10.minutes, only: :update, with: -> { too_many_attempts }

  before_action :no_store
  before_action :set_user_by_token, only: [:edit, :update]

  def new
  end

  def create
    if (user = User.find_by(email_address: params[:email_address].to_s.strip.downcase))
      deliver(PasswordsMailer.reset(user, locale: I18n.locale))
    end

    redirect_to login_path, notice: t("auth.reset.sent")
  end

  # The link from the e-mail. The token moves into the session and the
  # browser goes on to an address without it, so it doesn't stay in the
  # history or reach analytics.
  def edit
    return unless params[:token]

    session[:password_reset_token] = params[:token]
    redirect_to edit_password_path
  end

  def update
    if params[:password].blank?
      flash.now[:alert] = t("auth.reset.blank")
      render :edit, status: :unprocessable_entity
    elsif @user.update(password: params[:password], password_confirmation: params[:password_confirmation])
      session.delete(:password_reset_token)
      @user.sessions.destroy_all
      redirect_to login_path, notice: t("auth.reset.done")
    else
      flash.now[:alert] = @user.errors.full_messages.to_sentence
      render :edit, status: :unprocessable_entity
    end
  end

  private

  def set_user_by_token
    token = params[:token] || session[:password_reset_token]
    @user = User.find_by_password_reset_token(token) if token.present?
    return if @user

    session.delete(:password_reset_token)
    redirect_to new_password_path, alert: t("auth.reset.invalid")
  end

  def deliver(mail)
    mail.deliver_now
  rescue StandardError => e
    Rails.logger.error("[PasswordsMailer] #{e.class}: #{e.message}")
  end
end
