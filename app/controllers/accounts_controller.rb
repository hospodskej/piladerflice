class AccountsController < ApplicationController
  before_action :require_login

  def show
    @orders = current_user.orders.order(created_at: :desc).limit(10)
  end

  def edit
    @user = current_user
  end

  def update
    if current_user.update(profile_params)
      redirect_to account_path, notice: t("auth.profile_updated")
    else
      @user = current_user
      render :edit, status: :unprocessable_entity
    end
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

  def profile_params
    permitted = params.require(:user).permit(*(User::PROFILE_ATTRIBUTES - ["phone"]), :phone_prefix, :phone_number)
    phone_prefix = permitted.delete(:phone_prefix).presence_in(User::PHONE_PREFIXES) || User::PHONE_PREFIXES.first
    phone_number = permitted.delete(:phone_number).to_s.strip
    permitted[:phone] = phone_number.present? ? "#{phone_prefix} #{phone_number}" : nil
    permitted
  end
end
