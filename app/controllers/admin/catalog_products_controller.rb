module Admin
  class CatalogProductsController < BaseController
    before_action :set_catalog_product, only: [:edit, :update, :destroy]

    def index
      @catalog_products = CatalogProduct.ordered
    end

    def new
      @catalog_product = CatalogProduct.new
    end

    def create
      @catalog_product = CatalogProduct.new(catalog_product_params)

      if @catalog_product.save
        redirect_to edit_admin_catalog_product_path(@catalog_product), notice: "Produkt byl vytvořen. Nyní k němu přidejte varianty s cenami."
      else
        render :new, status: :unprocessable_entity
      end
    end

    def edit
    end

    def update
      if @catalog_product.update(catalog_product_params)
        redirect_to edit_admin_catalog_product_path(@catalog_product), notice: "Produkt byl uložen."
      else
        render :edit, status: :unprocessable_entity
      end
    end

    def destroy
      @catalog_product.destroy
      redirect_to admin_catalog_products_path, notice: "Produkt byl smazán."
    end

    private

    def set_catalog_product
      @catalog_product = CatalogProduct.find(params[:id])
    end

    def catalog_product_params
      params.require(:catalog_product).permit(
        :key, :template, :category, :hardness, :image, :active, :position,
        :title, :title_de, :type_label, :type_label_de,
        :subtitle, :subtitle_de, :description, :description_de,
        :drying_note, :drying_note_de, :image_alt, :image_alt_de
      )
    end
  end
end
