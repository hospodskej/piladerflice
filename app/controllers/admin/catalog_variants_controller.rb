module Admin
  class CatalogVariantsController < BaseController
    before_action :set_catalog_product
    before_action :set_catalog_variant, only: [:edit, :update, :destroy]

    def new
      @catalog_variant = @catalog_product.catalog_variants.new
    end

    def create
      @catalog_variant = @catalog_product.catalog_variants.new(catalog_variant_params)

      if @catalog_variant.save
        redirect_to edit_admin_catalog_product_path(@catalog_product), notice: "Varianta byla přidána."
      else
        render :new, status: :unprocessable_entity
      end
    end

    def edit
    end

    def update
      if @catalog_variant.update(catalog_variant_params)
        redirect_to edit_admin_catalog_product_path(@catalog_product), notice: "Varianta byla uložena."
      else
        render :edit, status: :unprocessable_entity
      end
    end

    def destroy
      @catalog_variant.destroy
      redirect_to edit_admin_catalog_product_path(@catalog_product), notice: "Varianta byla smazána."
    end

    private

    def set_catalog_product
      @catalog_product = CatalogProduct.find(params[:catalog_product_id])
    end

    def set_catalog_variant
      @catalog_variant = @catalog_product.catalog_variants.find(params[:id])
    end

    def catalog_variant_params
      params.require(:catalog_variant).permit(
        :key, :variant_group, :length_label, :variant_label, :variant_label_de,
        :amount_value, :price_czk, :in_stock, :position,
        :grade, :grade_de, :width_mm, :height_mm
      )
    end
  end
end
