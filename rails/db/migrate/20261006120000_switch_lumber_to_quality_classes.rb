class SwitchLumberToQualityClasses < ActiveRecord::Migration[8.1]
  class MigrationProduct < ActiveRecord::Base
    self.table_name = "catalog_products"
  end

  class MigrationVariant < ActiveRecord::Base
    self.table_name = "catalog_variants"
  end

  CLASS_I = { grade: "I. jakostní třída", grade_de: "Güteklasse I" }.freeze
  CLASS_II = { grade: "II. jakostní třída", grade_de: "Güteklasse II" }.freeze

  def up
    MigrationProduct.where(key: %w[tramy late fosny]).find_each do |product|
      product.update_columns(type_label: CLASS_I[:grade], type_label_de: CLASS_I[:grade_de])
      MigrationVariant.where(catalog_product_id: product.id, grade: [nil, ""]).update_all(CLASS_I)
    end

    prkna = MigrationProduct.find_by(key: "prkna")
    return unless prkna

    prkna.update_columns(title: "Prkna", title_de: "Bretter",
                         type_label: "I. a II. jakostní třída", type_label_de: "Güteklasse I und II")

    variants = MigrationVariant.where(catalog_product_id: prkna.id)
    unsorted_ids = variants.where(grade: "I. netříděné").pluck(:id)
    ActiveStorage::Attachment.where(record_type: "CatalogVariant", record_id: unsorted_ids).find_each(&:purge)
    MigrationVariant.where(id: unsorted_ids).delete_all

    variants.where(grade: "I. tříděné").update_all(CLASS_I)
    variants.where(grade: "II. tříděné").update_all(CLASS_II)
  end

  def down
    raise ActiveRecord::IrreversibleMigration
  end
end
