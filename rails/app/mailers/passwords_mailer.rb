class PasswordsMailer < ApplicationMailer
  # The link to set a new password ("Zapomenuté heslo").
  def reset(user, locale: I18n.default_locale)
    @user = user
    I18n.with_locale(locale) do
      @url = edit_password_url(token: user.password_reset_token, locale: locale == :de ? "at" : nil)
      mail(to: user.email_address, subject: t("auth.reset.mail_subject"))
    end
  end
end
