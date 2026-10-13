# The container grid on a firewood page: lengths across, amounts down.
#
# Loose wood (25, 33 cm) and stacked wood (50 cm, 1 m) have different top
# amounts (14 and 13 prm), so rows are not matched by value but by position:
# the n-th amount of each kind of wood shares a row, and the row is labelled
# with its amounts ("5", "10", "13 a 14"). Wood of one kind only gives the
# plain rows.
class FirewoodGrid
  def initialize(variants)
    @variants = variants.to_a
    @amounts_by_kind = @variants.group_by(&:stacked?).transform_values { |group| group.map(&:amount_value).uniq.compact.sort }
  end

  def lengths
    @variants.map(&:length_label).uniq
  end

  # One entry per row: the amounts in it, ascending, e.g. [13, 14].
  def rows
    row_count = @amounts_by_kind.values.map(&:size).max.to_i
    (0...row_count).map { |index| @amounts_by_kind.values.filter_map { |amounts| amounts[index] }.uniq.sort }
  end

  # The variant of this length in the given row, if there is one.
  def variant_at(row_index, length)
    @variants.find do |variant|
      variant.length_label == length && @amounts_by_kind[variant.stacked?][row_index] == variant.amount_value
    end
  end
end
