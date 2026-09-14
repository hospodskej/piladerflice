class CartItemsController < ApplicationController
  def create
    variant = CatalogVariant.joins(:catalog_product).where(catalog_products: { active: true }).find(params.require(:catalog_variant_id))

    @added_line_id = current_cart.add(variant)
    @added_item = current_cart.find(@added_line_id)

    respond_to do |format|
      format.turbo_stream
      format.html { redirect_back fallback_location: root_path }
    end
  end

  def update
    current_cart.update_quantity(params[:id], params.require(:quantity))
    respond_to do |format|
      format.turbo_stream
      format.html { redirect_to cart_path }
    end
  end

  def destroy
    current_cart.remove(params[:id])
    respond_to do |format|
      format.turbo_stream { render :update }
      format.html { redirect_to cart_path }
    end
  end
end
