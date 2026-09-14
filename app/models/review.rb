class Review < ApplicationRecord
  AVATAR_COLORS = [ "#4285F4", "#EA4335", "#9C27B0", "#0F9D58", "#F4B400", "#FF7043", "#00ACC1", "#8E24AA" ].freeze

  scope :ordered, -> { order(:position, :id) }

  def initial
    name.first.upcase
  end

  def avatar_color
    AVATAR_COLORS[id % AVATAR_COLORS.length]
  end

  def time_ago_i18n
    today = Date.current
    years = today.year - reviewed_on.year
    years -= 1 if today.month < reviewed_on.month || (today.month == reviewed_on.month && today.day < reviewed_on.day)

    if years >= 1
      relative_time(years, cs_singular: "rokem", cs_plural: "lety", de_singular: "Jahr", de_plural: "Jahren")
    else
      months = (today.year - reviewed_on.year) * 12 + (today.month - reviewed_on.month)
      months -= 1 if today.day < reviewed_on.day
      months = [ months, 1 ].max
      relative_time(months, cs_singular: "měsícem", cs_plural: "měsíci", de_singular: "Monat", de_plural: "Monaten")
    end
  end

  private

  def relative_time(count, cs_singular:, cs_plural:, de_singular:, de_plural:)
    if I18n.locale == :de
      count == 1 ? "vor 1 #{de_singular}" : "vor #{count} #{de_plural}"
    else
      count == 1 ? "před #{cs_singular}" : "před #{count} #{cs_plural}"
    end
  end
end
