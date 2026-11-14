class Inquiry < ApplicationRecord
  validates :first_name, :last_name, :email, :phone, presence: true
  validates :items_snapshot, presence: true
  validate :asks_for_something

  # The rows asked for, per category, in the form's order:
  # { "palivove" => [{ "druh" => "Dub", ... }], "stavebni" => [...] }.
  # Firewood and lumber can be asked for together.
  def items_by_category
    data = JSON.parse(items_snapshot.presence || "{}")
    return {} unless data.is_a?(Hash)

    InquiryFormOption::CATEGORIES.filter_map { |category| [category, data[category]] if data[category].present? }.to_h
  end

  def categories
    items_by_category.keys
  end

  def category_labels
    categories.map { |category| InquiryFormOption::CATEGORY_LABELS.fetch(category, category) }.join(" + ")
  end

  def full_name
    "#{first_name} #{last_name}".strip
  end

  def full_address
    [
      [street, house_number].compact_blank.join(" "),
      [zip, city].compact_blank.join(" ")
    ].compact_blank.join(", ")
  end

  private

  def asks_for_something
    errors.add(:items_snapshot, :blank) if items_by_category.empty?
  end
end
