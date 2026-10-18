//! The top of the home page: the claim, this week's promo, the Google
//! rating, why buy here, and the three main categories.

use leptos::prelude::*;

use crate::islands::widgets::FitLines;
use crate::ui::{asset, path, t};

/// This week's promo.
pub struct Promo {
    pub title: String,
    pub features: [String; 3],
    /// In the page's currency.
    pub price: String,
    pub link: String,
    pub image: Option<String>,
}

const FEATURES: [(&str, &str); 6] = [
    ("doprava", "doprava.webp"),
    ("vyroba", "vyroba.webp"),
    ("sortiment", "sortiment.webp"),
    ("vyrobce", "vyrobce.webp"),
    ("garance", "garance.webp"),
    ("ceny", "wallet.webp"),
];

const TOP_CATEGORIES: [(&str, &str, &str); 3] = [
    ("palivove_drevo", "top-palivove-drevo.webp", "/sortiment/palivove-drevo"),
    (
        "stavebni_drevo",
        "top-stavebni-drevo.webp",
        "/sortiment/stavebni-rezivo",
    ),
    ("vyrobni_zbytky", "top-vyrobni-zbytky.webp", "/sortiment/vyrobni-zbytky"),
];

#[component]
pub fn Hero(promo: Option<Promo>) -> impl IntoView {
    view! {
        <section class="hero-section">
            <div class="hero-intro">
                <div>
                    <FitLines min=20.0>
                        <span class="text-accent">{t("home.index.hero_title_accent")}</span>
                        <br/>
                        {t("home.index.hero_title_rest")}
                    </FitLines>
                    <p>{t("home.index.hero_text")}</p>
                </div>
                <div class="domu-buttons">
                    <a class="btn-eshop" href=path("/eshop/")>{t("home.index.btn_eshop")}</a>
                    " "
                    <a class="btn-vyroba" href=path("/kontakt#kalkulace")>{t("home.index.btn_vyroba")}</a>
                </div>
            </div>
            {promo.map(|promo| view! { <PromoCard promo=promo/> })}
            <div class="hero-trust">
                <Rating/>
                <div class="features">
                    {FEATURES
                        .map(|(key, image)| {
                            let name = t(&format!("home.index.features.{key}"));
                            view! {
                                <div class="feature-card">
                                    <div class="feature-item">
                                        <img alt=name.clone() src=asset(&format!("features/{image}"))/>
                                    </div>
                                    <span class="feature-text" inner_html=name></span>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
            <div class="top-categories-section">
                <h2 class="top-categories-title">
                    <span class="brand-blue">{t("home.index.top_categories_title_accent")}</span>
                    " "
                    {t("home.index.top_categories_title_rest")}
                </h2>
                <div class="top-categories-grid">
                    {TOP_CATEGORIES
                        .map(|(key, image, url)| {
                            let name = t(&format!("home.index.categories.{key}"));
                            view! {
                                <div class="top-categories-item">
                                    <a class="top-categories-image-link" href=path(url)>
                                        <img alt=name.clone() class="top-categories-image" src=asset(&format!("top-kategorie/{image}"))/>
                                    </a>
                                    <a class="top-categories-button" href=path(url)>{name}</a>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </section>
    }
}

#[component]
fn PromoCard(promo: Promo) -> impl IntoView {
    view! {
        <div class="akcni-product-card">
            <div class="badge">{t("home.index.promo_badge")}</div>
            <div class="akcni-media">
                {promo.image.map(|src| view! { <img alt=t("home.index.promo_alt") class="hero-image" src=src/> })}
                <div class="product-info">
                    <div class="product-subinfo">
                        <h2 class="product-title">{promo.title}</h2>
                        <ul class="product-features">
                            {promo.features.map(|feature| view! { <li>{feature}</li> }).collect_view()}
                        </ul>
                    </div>
                    <div class="akcni-product-price">
                        <span class="akcni-nabidka-price">{promo.price}</span>
                        " "
                        <a class="akce-btn-eshop" href=promo.link>{t("home.index.promo_cta")}</a>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Google's 4.6 stars.
#[component]
fn Rating() -> impl IntoView {
    view! {
        <div class="reviews">
            <img alt="Google recenze" class="google-logo" width="48" height="48" src=asset("google-review.webp")/>
            " "
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 370 48" width="280" height="48">
                <defs>
                    <g id="star">
                        <path fill="#FABB05" d="M24 34.54l12.36 7.46-3.28-14.06 10.92-9.46-14.38-1.22L24 4 18.38 17.26 4 18.48l10.92 9.46-3.28 14.06z"/>
                    </g>
                </defs>
                {[0, 60, 120, 180, 240].map(|x| view! { <use href="#star" x=x y="0"/> }).collect_view()}
            </svg>
            <div class="review-text">
                <strong>"4.6 "</strong>
                <span class="review-text-accent">{t("home.index.reviews_count")}</span>
            </div>
        </div>
    }
}
