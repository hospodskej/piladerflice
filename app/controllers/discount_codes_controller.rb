# The discount code box in the cart.
class DiscountCodesController < ApplicationController
  def create
    code = params[:code].to_s.strip

    if code.blank?
      reason = :blank
    else
      reason = current_cart.apply_discount(code)
    end

    message = reason ? helpers.discount_reason_message(reason, DiscountCode.lookup(code)) : t("cart.discount_applied", code: current_cart.discount.code)
    respond(message, error: reason.present?)
  end

  def destroy
    current_cart.remove_discount
    respond(nil)
  end

  private

  def respond(message, error: false)
    respond_to do |format|
      format.turbo_stream do
        @discount_message = message
        @discount_error = error
        render "cart_items/update"
      end
      format.html do
        flash[:discount_message] = message
        flash[:discount_error] = error
        redirect_to cart_path
      end
    end
  end
end
