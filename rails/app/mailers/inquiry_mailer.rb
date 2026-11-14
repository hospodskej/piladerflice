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
