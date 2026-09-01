class CartItemsController < ApplicationController
  # Adds one unit of a product/variant to the cart (or increments quantity
  # if that exact configuration is already in the cart - see Cart#add).
  # Renders a Turbo Stream that refreshes the header's cart widget and
  # shows the "added to cart" confirmation modal, so the page never
  # actually navigates away from wherever the "Do košíku" button was
  # clicked.
  def create
    specs = parsed_specs

    @added_line_id = current_cart.add(
      product_key: params.require(:product_key),
      image: params.require(:image),
      unit_price_czk: params.require(:unit_price_czk).to_i,
      specs: specs
    )
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

  private

  # `specs` arrives as a JSON-encoded string (see shared/_add_to_cart_form)
  # since it's an ordered array of small hashes describing the variant -
  # awkward to express as flat form fields, easy as one JSON blob.
  def parsed_specs
    raw = params.require(:specs)
    parsed = JSON.parse(raw)
    raise ActionController::BadRequest, "specs must be an array" unless parsed.is_a?(Array)

    parsed
  rescue JSON::ParserError
    raise ActionController::BadRequest, "specs must be valid JSON"
  end
end
