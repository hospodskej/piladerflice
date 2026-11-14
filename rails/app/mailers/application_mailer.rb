class ApplicationMailer < ActionMailer::Base
  default from: ENV.fetch("MAIL_FROM_ADDRESS", "objednavky@piladerflice.cz")
  layout "mailer"
end
