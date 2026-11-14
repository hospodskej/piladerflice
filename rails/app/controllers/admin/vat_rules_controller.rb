module Admin
  # VAT by product category (VatRule): whether admin prices include VAT, and
  # the rates for the Czech Republic and Austria.
  class VatRulesController < BaseController
    def index
      @vat_rules = rules
    end

    def update
      @vat_rules = rules
      rule = @vat_rules.find { |r| r.category == params[:id] } or raise ActiveRecord::RecordNotFound

      if rule.update(params.require(:vat_rule).permit(:prices_include_vat, :rate_cz, :rate_at))
        redirect_to admin_vat_rules_path, notice: "DPH pro „#{rule.label}“ bylo uloženo."
      else
        render :index, status: :unprocessable_entity
      end
    end

    private

    # Every category, with a row created for any that has none yet.
    def rules
      CatalogProduct::CATEGORIES.map do |category|
        VatRule.find_or_create_by!(category: category) do |rule|
          rule.prices_include_vat = true
          rule.rate_cz = VatRule::DEFAULT_RATES["cz"]
          rule.rate_at = VatRule::DEFAULT_RATES["at"]
        end
      end
    end
  end
end
