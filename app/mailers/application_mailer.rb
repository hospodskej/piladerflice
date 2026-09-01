class ApplicationMailer < ActionMailer::Base
  # If sending via Gmail SMTP (see config/environments/production.rb),
  # this MUST be the same address as SMTP_USERNAME, or Gmail will reject
  # the message - Gmail doesn't allow sending "from" an address other than
  # the authenticated account (or a verified alias of it). Other SMTP
  # providers are usually more flexible, but matching is the safe default.
  default from: ENV.fetch("MAIL_FROM_ADDRESS", "objednavky@piladerflice.cz")
  layout "mailer"
end
