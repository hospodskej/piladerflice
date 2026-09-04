class CheckoutController < ApplicationController
  before_action :redirect_if_cart_empty, except: [:confirmation]
  before_action :redirect_unless_shipping_selected, only: [:details, :update_details, :summary, :confirm]
  before_action :redirect_unless_details_complete, only: [:summary, :confirm]

  # GET /kosik/doprava - shipping method & payment method
  def shipping
  end

  def update_shipping
    current_checkout.update(shipping_params)
    redirect_to checkout_details_path
  end

  # GET /kosik/udaje - personal + billing (+ optional company / delivery) details
  def details
  end

  def update_details
    current_checkout.update(details_params)
    redirect_to checkout_summary_path
  end

  # GET /kosik/souhrn - final review before submitting
  def summary
    @order = Order.new(order_attributes)
  end

  # POST /kosik/souhrn - creates the order, emails it, empties the cart
  def confirm
    @order = Order.new(order_attributes)

    if params[:terms_agreement] != "1"
      @order.errors.add(:base, t("checkout.terms_required"))
      render :summary, status: :unprocessable_entity
      return
    end

    if @order.save
      deliver_order_notification(@order)
      deliver_customer_confirmation(@order)
      current_cart.clear
      current_checkout.clear
      session[:last_order_id] = @order.id
      redirect_to checkout_confirmation_path
    else
      render :summary, status: :unprocessable_entity
    end
  end

  # GET /kosik/dekujeme
  def confirmation
    @order = Order.find_by(id: session[:last_order_id])

    unless @order
      redirect_to cart_path
      return
    end
  end

  private

  def redirect_if_cart_empty
    redirect_to cart_path if current_cart.empty?
  end

  def redirect_unless_shipping_selected
    redirect_to checkout_shipping_path unless current_checkout.shipping_selected?
  end

  def redirect_unless_details_complete
    redirect_to checkout_details_path unless current_checkout.details_complete?
  end

  def shipping_params
    params.permit(:shipping_method, :payment_method, :country)
  end

  def details_params
    params.permit(
      :first_name, :last_name, :email, :phone, :newsletter_opt_in,
      :billing_street, :billing_city, :billing_zip, :billing_country,
      :company_purchase, :company_name, :company_ico, :company_dic,
      :delivery_address_different, :delivery_street, :delivery_city, :delivery_zip, :delivery_country
    )
  end

  # Builds an (unsaved) Order from everything gathered across the wizard so
  # far - used both to show the review on the summary page and, again, to
  # actually persist on final confirmation.
  def order_attributes
    {
      first_name: current_checkout["first_name"],
      last_name: current_checkout["last_name"],
      email: current_checkout["email"],
      phone: current_checkout["phone"],
      newsletter_opt_in: current_checkout.newsletter_opt_in?,
      billing_street: current_checkout["billing_street"],
      billing_city: current_checkout["billing_city"],
      billing_zip: current_checkout["billing_zip"],
      billing_country: current_checkout["billing_country"],
      company_purchase: current_checkout.company_purchase?,
      company_name: current_checkout["company_name"],
      company_ico: current_checkout["company_ico"],
      company_dic: current_checkout["company_dic"],
      delivery_address_different: current_checkout.delivery_address_different?,
      delivery_street: current_checkout["delivery_street"],
      delivery_city: current_checkout["delivery_city"],
      delivery_zip: current_checkout["delivery_zip"],
      delivery_country: current_checkout["delivery_country"],
      shipping_method: current_checkout["shipping_method"],
      payment_method: current_checkout["payment_method"],
      items_snapshot: cart_items_snapshot,
      subtotal_czk: current_cart.subtotal_czk,
      vat_czk: current_cart.subtotal_czk - current_cart.subtotal_excl_vat_czk,
      total_czk: current_cart.subtotal_czk,
      locale: I18n.locale.to_s
    }
  end

  def cart_items_snapshot
    current_cart.items.map do |item|
      {
        product_key: item.product_key,
        image: item.image,
        unit_price_czk: item.unit_price_czk,
        specs: item.specs,
        quantity: item.quantity
      }
    end.to_json
  end

  # Emailing is best-effort: the order is already safely saved in the
  # database by this point (see #confirm), so a temporary SMTP problem
  # shouldn't stop the customer from reaching the confirmation page - it
  # just means the instant email notification didn't go out for this one
  # order. Logged loudly either way so it isn't silently missed.
  def deliver_order_notification(order)
    I18n.with_locale(:cs) { OrderMailer.new_order(order).deliver_now }
  rescue StandardError => e
    Rails.logger.error("[OrderMailer] failed to send business notification for order ##{order.id}: #{e.class}: #{e.message}")
  end

  # Independent from deliver_order_notification - if this one fails, the
  # business still got their copy and the order still exists, so a
  # customer-side delivery hiccup doesn't lose the order either way.
  def deliver_customer_confirmation(order)
    I18n.with_locale(order.locale) { OrderMailer.customer_confirmation(order).deliver_now }
  rescue StandardError => e
    Rails.logger.error("[OrderMailer] failed to send customer confirmation for order ##{order.id}: #{e.class}: #{e.message}")
  end
end
