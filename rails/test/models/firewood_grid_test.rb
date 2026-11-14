require "test_helper"

class FirewoodGridTest < ActiveSupport::TestCase
  def variant(length, amount, stacked:)
    CatalogVariant.new(variant_group: "kontejner", length_label: length, amount_value: amount, stacked: stacked)
  end

  def standard_variants
    [
      variant("100 cm", 5, stacked: true), variant("50 cm", 5, stacked: true),
      variant("33 cm", 5, stacked: false), variant("25 cm", 5, stacked: false),
      variant("100 cm", 10, stacked: true), variant("50 cm", 10, stacked: true),
      variant("33 cm", 10, stacked: false), variant("25 cm", 10, stacked: false),
      variant("100 cm", 13, stacked: true), variant("50 cm", 13, stacked: true),
      variant("33 cm", 14, stacked: false), variant("25 cm", 14, stacked: false)
    ]
  end

  test "loose and stacked wood with different top amounts share a row, labelled with both" do
    grid = FirewoodGrid.new(standard_variants)

    assert_equal [[5], [10], [13, 14]], grid.rows
    assert_equal ["100 cm", "50 cm", "33 cm", "25 cm"], grid.lengths
  end

  test "every row has a card in every column: no empty cells" do
    grid = FirewoodGrid.new(standard_variants)

    grid.rows.each_index do |row|
      cells = grid.lengths.map { |length| grid.variant_at(row, length) }
      assert cells.all?, "row #{row} has an empty cell"
    end
  end

  test "each card keeps its own amount" do
    grid = FirewoodGrid.new(standard_variants)

    assert_equal 13, grid.variant_at(2, "100 cm").amount_value
    assert_equal 13, grid.variant_at(2, "50 cm").amount_value
    assert_equal 14, grid.variant_at(2, "33 cm").amount_value
    assert_equal 14, grid.variant_at(2, "25 cm").amount_value
  end

  test "equal amounts give plain rows" do
    variants = [variant("100 cm", 5, stacked: true), variant("33 cm", 5, stacked: false),
                variant("100 cm", 10, stacked: true), variant("33 cm", 10, stacked: false)]

    assert_equal [[5], [10]], FirewoodGrid.new(variants).rows
  end

  test "one kind of wood only gives plain rows" do
    variants = [variant("33 cm", 5, stacked: false), variant("25 cm", 10, stacked: false), variant("33 cm", 14, stacked: false)]
    grid = FirewoodGrid.new(variants)

    assert_equal [[5], [10], [14]], grid.rows
    assert_nil grid.variant_at(1, "33 cm"), "no 33 cm variant in the 10 row"
    assert_equal 10, grid.variant_at(1, "25 cm").amount_value
  end

  test "kinds with a different number of rows leave the extra rows to one kind" do
    variants = [variant("100 cm", 5, stacked: true), variant("33 cm", 5, stacked: false), variant("33 cm", 10, stacked: false)]
    grid = FirewoodGrid.new(variants)

    assert_equal [[5], [10]], grid.rows
    assert_nil grid.variant_at(1, "100 cm")
  end

  test "no variants, no rows" do
    grid = FirewoodGrid.new([])

    assert_equal [], grid.rows
    assert_equal [], grid.lengths
  end
end
