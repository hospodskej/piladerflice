class SetContainerCapacitiesTo13And14Prm < ActiveRecord::Migration[8.1]
  class MigrationVariant < ActiveRecord::Base
    self.table_name = "catalog_variants"
  end

  # A container holds at most 14 prm of loose wood (25, 33 cm) and 13 prm of
  # stacked wood (50 cm, 1 m), so the top "15 prm" option becomes 14 / 13.
  def up
    containers = MigrationVariant.where(variant_group: "kontejner", amount_value: 15)
    containers.where(stacked: true).update_all(amount_value: 13)
    containers.where(stacked: false).update_all(amount_value: 14)
  end

  def down
    MigrationVariant.where(variant_group: "kontejner", stacked: true, amount_value: 13).update_all(amount_value: 15)
    MigrationVariant.where(variant_group: "kontejner", stacked: false, amount_value: 14).update_all(amount_value: 15)
  end
end
