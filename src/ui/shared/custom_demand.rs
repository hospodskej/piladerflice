//! "Something else? We make it to measure": the banner to the quote form.

use leptos::prelude::*;

use crate::ui::{asset, path, t, t_html};

#[component]
pub fn CustomDemand() -> impl IntoView {
    view! {
        <section class="custom-demand-section">
            <h2 class="custom-demand-title">
                {t("shared.custom_demand.title_line1")}
                " "
                <span class="text-blue">{t("shared.custom_demand.title_line2")}</span>
            </h2>
            <div class="custom-demand-card">
                <div class="custom-demand-img-wrapper">
                    <img alt=t("shared.custom_demand.banner_alt") class="custom-demand-banner-img" src=asset("eshop/custom-production-banner.webp")/>
                </div>
                <div class="custom-demand-content">
                    <div class="custom-demand-text">
                        <h3>{t("shared.custom_demand.card_title")}</h3>
                        <p inner_html=t_html("shared.custom_demand.text_html")></p>
                    </div>
                    <div class="custom-demand-action">
                        <a class="icon-only-btn" href=path("/kontakt#kalkulace")>
                            <svg viewBox="0 0 24 24" width="30" height="30" fill="white">
                                <path d="M8.59 16.59L13.17 12 8.59 7.41 10 6l6 6-6 6-1.41-1.41z"/>
                            </svg>
                        </a>
                    </div>
                </div>
            </div>
        </section>
    }
}
