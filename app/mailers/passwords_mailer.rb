class PasswordsMailer < ApplicationMailer
  # The link to set a new password ("Zapomenuté heslo").
  def reset(user, locale: I18n.default_locale)
    @user = user
    I18n.with_locale(locale) do
      @url = edit_password_url(token: user.password_reset_token, locale: locale == :de ? "at" : nil)
      mail(to: user.email_address, subject: t("auth.reset.mail_subject"))
    end
  end

  # Someone tried to register with an e-mail that already has an account.
  # The registration page doesn't say so (that would show who is
  # registered); the owner learns it here, with the way back in.
  def account_exists(user, locale: I18n.default_locale)
    @user = user
    I18n.with_locale(locale) do
      @login_url = login_url(locale: locale == :de ? "at" : nil)
      @reset_url = new_password_url(locale: locale == :de ? "at" : nil)
      mail(to: user.email_address, subject: t("auth.account_exists.mail_subject"))
    end
  end
end
