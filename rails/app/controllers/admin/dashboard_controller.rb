module Admin
  class DashboardController < BaseController
    def index
      @recent_orders = Order.order(created_at: :desc).limit(5)
      @catalog_product_count = CatalogProduct.count
      @order_count = Order.count
      @orders_this_month = Order.where(created_at: Time.current.beginning_of_month..).count
      @orders_last_month = Order.where(created_at: 1.month.ago.beginning_of_month..1.month.ago.end_of_month).count
    end
  end
end
