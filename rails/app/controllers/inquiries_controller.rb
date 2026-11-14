class InquiriesController < ApplicationController
  # Every inquiry sends an e-mail.
  rate_limit to: 5, within: 10.minutes, only: :create, with: -> { too_many_attempts }

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

  # The form sends each category's rows column by column, e.g.
  # items[palivove][druh][] = ["Dub", "Buk"]. A category switched off in the
  # form isn't sent at all.
  def items_snapshot
    submitted = params[:items].respond_to?(:to_unsafe_h) ? params[:items].to_unsafe_h : {}

    InquiryFormOption::CATEGORIES.each_with_object({}) do |category, snapshot|
      next unless submitted[category].is_a?(Hash)

      fields = InquiryFormOption::ROW_FIELDS.fetch(category)
      columns = fields.index_with { |field| Array(submitted[category][field]).map(&:to_s) }
      row_count = columns.values.map(&:size).max.to_i
      rows = (0...row_count).map { |i| fields.index_with { |field| columns[field][i] } }
      snapshot[category] = rows if rows.any?
    end.to_json
  end

  def deliver_inquiry_notification(inquiry)
    I18n.with_locale(:cs) { InquiryMailer.new_inquiry(inquiry).deliver_now }
  rescue StandardError => e
    Rails.logger.error("[InquiryMailer] failed to send notification for inquiry ##{inquiry.id}: #{e.class}: #{e.message}")
  end
end
