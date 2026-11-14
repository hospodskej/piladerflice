module Admin
  class PromosController < BaseController
    before_action :set_promo, only: [:edit, :update, :destroy]

    def index
      @promos = Promo.order(:id)
      @current_weekly_pick = Promo.weekly_pick
    end

    def new
      @promo = Promo.new
    end

    def create
      @promo = Promo.new(promo_params)

      if @promo.save
        redirect_to admin_promos_path, notice: "Doporučení bylo vytvořeno."
      else
        render :new, status: :unprocessable_entity
      end
    end

    def edit
    end

    def update
      if @promo.update(promo_params)
        redirect_to admin_promos_path, notice: "Doporučení bylo uloženo."
      else
        render :edit, status: :unprocessable_entity
      end
    end

    def destroy
      @promo.destroy
      redirect_to admin_promos_path, notice: "Doporučení bylo smazáno."
    end

    private

    def set_promo
      @promo = Promo.find(params[:id])
    end

    def promo_params
      params.require(:promo).permit(
        :title, :feature_1, :feature_2, :feature_3, :price, :image, :link,
        :title_de, :feature_1_de, :feature_2_de, :feature_3_de, :price_de
      )
    end
  end
end
