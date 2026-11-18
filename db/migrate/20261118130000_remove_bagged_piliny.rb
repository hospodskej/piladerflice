# Štěpán: piliny are sold loose only, so the "Balené" variant goes, and the
# description stops offering bags. Only touches the seeded variant and the
# seeded sentence, so anything edited in the admin stays as it is. Orders
# keep their own copy of what was bought.
class RemoveBaggedPiliny < ActiveRecord::Migration[8.1]
  OLD_CS = "Nabízíme volně ložené nebo balené varianty."
  NEW_CS = "Prodáváme volně ložené."
  OLD_DE = "Erhältlich lose oder verpackt."
  NEW_DE = "Erhältlich lose geschüttet."

  class Product < ActiveRecord::Base
    self.table_name = "catalog_products"
  end

  class Variant < ActiveRecord::Base
    self.table_name = "catalog_variants"
  end

  def up
    product = Product.find_by(key: "piliny")
    return unless product

    Variant.where(catalog_product_id: product.id, key: "balene").find_each do |variant|
      execute "DELETE FROM active_storage_attachments WHERE record_type = 'CatalogVariant' AND record_id = #{variant.id.to_i}"
      variant.delete
    end

    product.update_columns(
      description: product.description.to_s.sub(OLD_CS, NEW_CS),
      description_de: product.description_de.to_s.sub(OLD_DE, NEW_DE)
    )
  end

  def down
    # The variant isn't brought back: add it again in Admin > Katalog a ceny if needed.
  end
end
