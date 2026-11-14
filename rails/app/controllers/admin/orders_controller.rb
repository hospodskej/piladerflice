module Admin
  class OrdersController < BaseController
    before_action :set_order, only: [:show, :destroy]

    def index
      @orders = Order.order(created_at: :desc)
    end

    def show
      @items = @order.items.map.with_index { |item, idx| CartLineItem.new(idx.to_s, item) }
    end

    def destroy
      @order.destroy
      redirect_to admin_orders_path, notice: "Objednávka byla smazána."
    end

    private

    def set_order
      @order = Order.find(params[:id])
    end
  end
end
