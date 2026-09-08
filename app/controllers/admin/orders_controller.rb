module Admin
  class OrdersController < BaseController
    def index
      @orders = Order.order(created_at: :desc)
    end

    def show
      @order = Order.find(params[:id])
      @items = @order.items.map.with_index { |item, idx| CartLineItem.new(idx.to_s, item) }
    end
  end
end
