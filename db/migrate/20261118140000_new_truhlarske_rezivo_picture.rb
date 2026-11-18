# The "Truhlářské řezivo" card in Sortiment showed a photo of fošny
# (construction lumber); it gets a photo of joinery lumber. Only the card
# still on the old seeded picture is changed.
class NewTruhlarskeRezivoPicture < ActiveRecord::Migration[8.1]
  def up
    execute <<~SQL
      UPDATE products SET image = 'sortiment/truhlarske-rezivo.webp'
      WHERE link = '/sortiment/truhlarske-rezivo' AND image = 'sortiment/fosny1.webp'
    SQL
  end

  def down
    execute <<~SQL
      UPDATE products SET image = 'sortiment/fosny1.webp'
      WHERE link = '/sortiment/truhlarske-rezivo' AND image = 'sortiment/truhlarske-rezivo.webp'
    SQL
  end
end
