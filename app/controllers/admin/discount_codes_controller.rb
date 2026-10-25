module Admin
  class DiscountCodesController < BaseController
    before_action :set_discount_code, only: [:edit, :update, :destroy]

    def index
      @discount_codes = DiscountCode.order(created_at: :desc)
    end

    def new
      @discount_code = DiscountCode.new
    end

    def create
      @discount_code = DiscountCode.new(discount_code_params)

      if @discount_code.save
        redirect_to admin_discount_codes_path, notice: "Slevový kód #{@discount_code.code} byl vytvořen."
      else
        render :new, status: :unprocessable_entity
      end
    end

    def edit
    end

    def update
      if @discount_code.update(discount_code_params)
        redirect_to admin_discount_codes_path, notice: "Slevový kód #{@discount_code.code} byl uložen."
      else
        render :edit, status: :unprocessable_entity
      end
    end

    def destroy
      @discount_code.destroy
      redirect_to admin_discount_codes_path, notice: "Slevový kód #{@discount_code.code} byl smazán."
    end

    private

    def set_discount_code
      @discount_code = DiscountCode.find(params[:id])
    end

    def discount_code_params
      params.require(:discount_code).permit(:code, :kind, :value, :active, :valid_until, :usage_limit, :min_order_czk, :note)
    end
  end
end
