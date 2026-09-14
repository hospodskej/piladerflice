class InquiryFormOption < ApplicationRecord
  include Translatable
  translates :value

  CATEGORIES = %w[palivove stavebni].freeze

  DROPDOWN_FIELDS = {
    "palivove" => %w[varianta druh delka mnozstvi],
    "stavebni" => %w[polozka delka vyska sirka]
  }.freeze

  ROW_FIELDS = {
    "palivove" => DROPDOWN_FIELDS["palivove"],
    "stavebni" => DROPDOWN_FIELDS["stavebni"] + %w[mnozstvi]
  }.freeze

  CATEGORY_LABELS = {
    "palivove" => "Palivové dřevo",
    "stavebni" => "Stavební řezivo"
  }.freeze

  FIELD_LABELS = {
    "varianta" => "Varianta",
    "druh" => "Druh",
    "delka" => "Délka",
    "mnozstvi" => "Množství",
    "polozka" => "Položka",
    "vyska" => "Výška",
    "sirka" => "Šířka"
  }.freeze

  validates :category, inclusion: { in: CATEGORIES }
  validates :value, presence: true
  validate :field_belongs_to_category

  scope :ordered, -> { order(:position, :id) }
  scope :for_field, ->(category, field) { where(category: category, field: field).ordered }

  private

  def field_belongs_to_category
    return if DROPDOWN_FIELDS.fetch(category, []).include?(field)

    errors.add(:field, :invalid_for_category)
  end
end
