module Admin
  # The delivery price per kilometre. It drives the checkout's delivery
  # formula and every text that quotes it (e-shop note, checkout, terms).
  class DeliverySettingsController < BaseController
    def edit
      @setting = delivery_setting
    end

    def update
      @setting = delivery_setting
      @setting.value = params.require(:setting)[:value].to_s.strip

      if @setting.save
        redirect_to edit_admin_delivery_settings_path, notice: "Cena dopravy byla uložena."
      else
        render :edit, status: :unprocessable_entity
      end
    end

    private

    def delivery_setting
      Setting.find_or_initialize_by(key: "delivery_czk_per_km") { |s| s.value = Setting.fetch("delivery_czk_per_km") }
    end
  end
end
