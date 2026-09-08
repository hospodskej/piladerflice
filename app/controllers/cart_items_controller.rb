class CartItemsController < ApplicationController
  # Adds one unit of a catalog variant to the cart (or increments quantity
  # if it's already in the cart - see Cart#add). The only thing trusted
  # from the request is which variant was chosen (its id) - price, image,
  # and description all come from the CatalogVariant record itself, never
  # from client-submitted form data, so there's nothing here for someone
  # to tamper with to change what they're charged.
  #
  # Renders a Turbo Stream that refreshes the header's cart widget and
  # shows the "added to cart" confirmation modal, so the page never
  # actually navigates away from wherever the "Do košíku" button was
  # clicked.
  def create
    variant = CatalogVariant.joins(:catalog_product).where(catalog_products: { active: true }).find(params.require(:catalog_variant_id))

    @added_line_id = current_cart.add(variant)
    @added_item = current_cart.find(@added_line_id)

    respond_to do |format|
      format.turbo_stream
      format.html { redirect_back fallback_location: root_path }
    end
  end

  # Updates a line's quantity (the stepper on the cart contents page).
  # Renders a Turbo Stream that refreshes both the cart page's item list
  # and totals, and the header's cart widget, so the displayed price
  # always matches what's actually in the cart.
  def update
    current_cart.update_quantity(params[:id], params.require(:quantity))
    respond_to do |format|
      format.turbo_stream
      format.html { redirect_to cart_path }
    end
  end

  # Removes a line entirely (the "×" button on the cart contents page).
  def destroy
    current_cart.remove(params[:id])
    respond_to do |format|
      format.turbo_stream { render :update }
      format.html { redirect_to cart_path }
    end
  end
end
