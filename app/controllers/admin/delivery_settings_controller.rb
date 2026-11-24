module Admin
  # The delivery price per kilometre. It drives the checkout's delivery
  # formula and every text that quotes it (e-shop note, checkout, terms).
  class DeliverySettingsController < BaseController
    def edit
      @setting = delivery_setting
      @eur_setting = eur_setting
    end

    def update
      @setting = delivery_setting
      @setting.value = params.require(:setting)[:value].to_s.strip
      @eur_setting = eur_setting
      @eur_setting.value = params.require(:setting)[:value_eur].to_s.strip.tr(",", ".")

      if @setting.valid? && @eur_setting.valid? && Setting.transaction { @setting.save! && @eur_setting.save! }
        redirect_to edit_admin_delivery_settings_path, notice: "Cena dopravy byla uložena."
      else
        render :edit, status: :unprocessable_entity
      end
    end

    private

    def delivery_setting
      Setting.find_or_initialize_by(key: "delivery_czk_per_km") { |s| s.value = Setting.fetch("delivery_czk_per_km") }
    end

    # The € price per km for the Austrian site; empty = converted from Kč.
    def eur_setting
      Setting.find_or_initialize_by(key: "delivery_eur_per_km") { |s| s.value = "" }
    end
  end
end
