class Inquiry < ApplicationRecord
  validates :first_name, :last_name, :email, :phone, presence: true
  validates :items_snapshot, presence: true

  def items
    JSON.parse(items_snapshot)
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
end
