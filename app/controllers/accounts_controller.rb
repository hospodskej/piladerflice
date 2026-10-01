class AccountsController < ApplicationController
  before_action :require_login

  def show
    @orders = current_user.orders.order(created_at: :desc).limit(10)
  end

  def reorder
    order = current_user.orders.find(params[:id])
    added = 0
    skipped = 0

    order.items.each do |item|
      variant = CatalogVariant.joins(:catalog_product)
                              .where(catalog_products: { active: true })
                              .find_by(id: item["catalog_variant_id"])

      if variant
        current_cart.add(variant, item["quantity"])
        added += 1
      else
        skipped += 1
      end
    end

    if added.zero?
      redirect_to account_path, alert: t("auth.reorder_failed")
    elsif skipped.positive?
      redirect_to cart_path, notice: t("auth.reorder_partial", count: skipped)
    else
      redirect_to cart_path, notice: t("auth.reorder_success")
    end
  end

  private

  def require_login
    return if current_user

    session[:return_to_after_authenticating] = request.url
    redirect_to login_path, alert: "Pro zobrazení účtu se musíte přihlásit."
  end
end
