module Admin
  class DashboardController < BaseController
    def index
      @recent_orders = Order.order(created_at: :desc).limit(5)
      @catalog_product_count = CatalogProduct.count
    end
  end
end
