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
      category: category,
      items_snapshot: items_snapshot,
      locale: I18n.locale.to_s
    }
  end

  def category
    InquiryFormOption::CATEGORIES.include?(params[:kalkulace_category]) ? params[:kalkulace_category] : InquiryFormOption::CATEGORIES.first
  end

  def items_snapshot
    fields = InquiryFormOption::ROW_FIELDS.fetch(category, [])
    columns = fields.index_with { |field| Array(params[field]) }
    row_count = columns.values.map(&:size).max.to_i

    (0...row_count).map do |i|
      fields.index_with { |field| columns[field][i] }
    end.to_json
  end

  def deliver_inquiry_notification(inquiry)
    I18n.with_locale(:cs) { InquiryMailer.new_inquiry(inquiry).deliver_now }
  rescue StandardError => e
    Rails.logger.error("[InquiryMailer] failed to send notification for inquiry ##{inquiry.id}: #{e.class}: #{e.message}")
  end
end
