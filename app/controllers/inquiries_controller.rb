class InquiriesController < ApplicationController
  def create
    @inquiry = Inquiry.new(inquiry_attributes)

    if @inquiry.save
      deliver_inquiry_notification(@inquiry)
      redirect_to kontakt_path(anchor: "kalkulace"), notice: t("kontakt.inquiry_success")
    else
      redirect_to kontakt_path(anchor: "kalkulace"), alert: t("kontakt.inquiry_error")
    end
  end

  private

  def inquiry_attributes
    {
      first_name: params[:jmeno],
      last_name: params[:prijmeni],
      email: params[:email],
      phone: params[:telefon],
      street: params[:ulice],
      house_number: params[:cislo_popisne],
      city: params[:mesto],
      zip: params[:psc],
      notes: params[:poznamky],
      items_snapshot: items_snapshot,
      locale: I18n.locale.to_s
    }
  end

  def items_snapshot
    varianta = Array(params[:varianta])
    druh = Array(params[:druh])
    delka = Array(params[:delka])
    mnozstvi = Array(params[:mnozstvi])

    varianta.each_index.map do |i|
      { "varianta" => varianta[i], "druh" => druh[i], "delka" => delka[i], "mnozstvi" => mnozstvi[i] }
    end.to_json
  end

  def deliver_inquiry_notification(inquiry)
    I18n.with_locale(:cs) { InquiryMailer.new_inquiry(inquiry).deliver_now }
  rescue StandardError => e
    Rails.logger.error("[InquiryMailer] failed to send notification for inquiry ##{inquiry.id}: #{e.class}: #{e.message}")
  end
end
