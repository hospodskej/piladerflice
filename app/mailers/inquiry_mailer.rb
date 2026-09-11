# The contact page's "kalkulace" quote-request form never had a backend at
# all before this - clicking submit did nothing. This mailer is the
# notification half of fixing that (see InquiriesController#create).
# Always written in Czech, same reasoning as OrderMailer: it's going to the
# business owner for follow-up, not the customer.
class InquiryMailer < ApplicationMailer
  NOTIFICATION_RECIPIENT = ENV.fetch("INQUIRY_NOTIFICATION_EMAIL", ENV.fetch("ORDER_NOTIFICATION_EMAIL", "pavelpatockaa@gmail.com"))

  def new_inquiry(inquiry)
    @inquiry = inquiry
    mail(
      to: NOTIFICATION_RECIPIENT,
      subject: "Nová poptávka ##{inquiry.id} – #{inquiry.full_name}"
    )
  end
end
