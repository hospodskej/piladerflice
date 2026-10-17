//! The home page: weekly promo, bestsellers, categories, services, the
//! "might interest you" slides, frequent questions and Google reviews.

mod hero;
mod info;
mod reviews;

pub use hero::Promo;
pub use info::{ServiceItem, Slide};
pub use reviews::Review;

use leptos::prelude::*;

use super::shared::CategoryTile;
use super::shared::{HowToOrder, ProductCard};
use super::{asset, path, t, t_html};

/// Everything the home page shows that comes from the database.
pub struct HomeData {
    pub promo: Option<Promo>,
    pub bestsellers: Vec<ProductCard>,
    pub tiles: Vec<CategoryTile>,
    pub services: Vec<ServiceItem>,
    pub slides: Vec<Slide>,
    pub reviews: Vec<Review>,
}

#[component]
pub fn HomePage(data: HomeData) -> impl IntoView {
    let HomeData {
        promo,
        bestsellers,
        tiles,
        services,
        slides,
        reviews,
    } = data;
    view! {
        <main class="main">
            <hero::Hero promo=promo/>
            <Bestsellers products=bestsellers/>
            <Offer/>
            <Tiles tiles=tiles/>
            <info::Services services=services/>
            <Delivery/>
            <News/>
            <section class="how-to-order-section">
                <HowToOrder/>
            </section>
            <info::Slides slides=slides/>
            <info::Faq/>
            <AboutUs/>
            <section class="let-zkusenosti-section">
                <div class="let-zkusenosti-banner">
                    <h2>{t("home.index.zkusenosti_banner")}</h2>
                </div>
            </section>
            <reviews::GoogleReviews reviews=reviews/>
        </main>
    }
}

/// A heading with an accented part: "Nejprodávanější **produkty**".
#[component]
pub fn Accented(rest: &'static str, accent: &'static str, #[prop(optional)] accent_first: bool) -> impl IntoView {
    let accent = view! { <span style="color: #2C8FFF">{t(accent)}</span> };
    if accent_first {
        view! { {accent}" "{t(rest)} }.into_any()
    } else {
        view! { {t(rest)}" "{accent} }.into_any()
    }
}

#[component]
fn Bestsellers(products: Vec<ProductCard>) -> impl IntoView {
    view! {
        <section class="nejprodavanejsi-produkty-section">
            <h1>
                <Accented accent="home.index.nejprodavanejsi_title_accent" rest="home.index.nejprodavanejsi_title_rest" accent_first=true/>
            </h1>
            <div class="nejprodavanejsi-produkty-grid">
                {products
                    .into_iter()
                    .map(|product| {
                        let href = path(&format!("/eshop/{}", product.key));
                        view! {
                            <div class="nejprodavanejsi-produkty-card">
                                <a class="nejprodavanejsi-produkty-image-link" data-turbo="false" href=href.clone()>
                                    {product
                                        .image
                                        .map(|src| view! { <img alt=product.title.clone() class="nejprodavanejsi-produkty-image" src=src/> })}
                                </a>
                                <div class="nejprodavanejsi-produkty-details">
                                    <span class="nejprodavanejsi-produkty-title">{product.type_label}</span>
                                    <span class="nejprodavanejsi-produkty-desc"><strong>{product.title}</strong></span>
                                    <span class="nejprodavanejsi-produkty-stock">{t("common.stock")}</span>
                                    <span class="nejprodavanejsi-produkty-price">
                                        <strong>{product.price_from.unwrap_or_else(|| "---".into())}</strong>
                                    </span>
                                    <a class="btn-prohlednout" data-turbo="false" href=href>{t("home.index.btn_prohlednout")}</a>
                                </div>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

/// "Akční nabídka": the price list teaser.
#[component]
fn Offer() -> impl IntoView {
    view! {
        <section class="akcni-nabidka-section">
            <div class="akcni-nabidka-left">
                <div class="akcni-nabidka-header">
                    <h1><Accented rest="home.index.akcni_nabidka_title_rest" accent="home.index.akcni_nabidka_title_accent"/></h1>
                    <div class="akcni-nabidka">
                        <h4 inner_html=t_html("home.index.akcni_nabidka_lead_html")></h4>
                        <h4 inner_html=t_html("home.index.akcni_nabidka_text_html")></h4>
                    </div>
                    <div class="akcni-nabidka-cenik">
                        <a class="akcni-nabidka-button" href=path("/kontakt#cenik")>{t("common.cenik")}</a>
                    </div>
                </div>
            </div>
            <div class="akcni-nabidka-right">
                <div class="akcni-nabidka-decoration">
                    <div class="product-image-wrapper">
                        <div class="akcni-badge">{t("home.index.akcni_badge")}</div>
                        <img alt=t("home.index.akcni_image_alt") class="akcni-nabidka-image" src=asset("akcni-nabidka.webp")/>
                    </div>
                </div>
            </div>
        </section>
    }
}

/// The Sortiment categories.
#[component]
fn Tiles(tiles: Vec<CategoryTile>) -> impl IntoView {
    view! {
        <section class="nabidka-section">
            <div class="nabidka-header">
                <h2><Accented rest="home.index.nabidka_title_rest" accent="home.index.nabidka_title_accent"/></h2>
                <div class="nabidka-text">
                    <p inner_html=t_html("home.index.nabidka_text_1_html")></p>
                    <p inner_html=t_html("home.index.nabidka_text_2_html")></p>
                </div>
            </div>
            <div class="nabidka-grid">
                {tiles
                    .into_iter()
                    .map(|tile| {
                        view! {
                            <div class="nabidka-card">
                                <a class="nabidka-image-link" href=tile.link.clone()>
                                    <img alt=tile.title.clone() class="nabidka-image" src=tile.image.as_deref().map(asset)/>
                                </a>
                                <a class="nabidka-btn" href=tile.link>{tile.title}</a>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

/// "We deliver": the delivery area map.
#[component]
fn Delivery() -> impl IntoView {
    let pin_label = t("home.index.dorucujeme_pin_label");
    let pin_address = t("home.index.dorucujeme_pin_address");
    view! {
        <section class="dorucujeme-section">
            <div class="dorucujeme-text">
                <h2><Accented rest="home.index.dorucujeme_title_rest" accent="home.index.dorucujeme_title_accent"/></h2>
                <p>{t("home.index.dorucujeme_text_1")}</p>
                <p>{t("home.index.dorucujeme_text_2")}</p>
                <a class="btn-primary" href=path("/kontakt#cenik")>{t("common.cenik")}</a>
            </div>
            <div class="dorucujeme-map">
                <div class="dorucujeme-map-inner">
                    <img alt=t("home.index.dorucujeme_map_alt") src=asset("mapa-doruceni.webp")/>
                    <div class="dorucujeme-pin" tabindex="0" aria-label=format!("{pin_label}: {pin_address}")>
                        <span class="dorucujeme-pin-tooltip">
                            <strong>{pin_label.clone()}</strong>
                            " "
                            {pin_address.clone()}
                        </span>
                    </div>
                </div>
            </div>
        </section>
    }
}

/// "Co je nového": three photos and a text.
#[component]
fn News() -> impl IntoView {
    view! {
        <section class="co-je-noveho-section">
            <div class="co-je-noveho-gallery">
                <img alt=t("home.index.co_je_noveho_pic1_alt") class="co-je-noveho-img co-je-noveho-img-main" src=asset("co-je-noveho-pic1.webp")/>
                <img alt=t("home.index.co_je_noveho_pic2_alt") class="co-je-noveho-img" src=asset("co-je-noveho-pic2.webp")/>
                <img alt=t("home.index.co_je_noveho_pic3_alt") class="co-je-noveho-img" src=asset("co-je-noveho-pic3.webp")/>
            </div>
            <div class="co-je-noveho-text">
                <h2><Accented rest="home.index.co_je_noveho_title_rest" accent="home.index.co_je_noveho_title_accent"/></h2>
                <h3>{t("home.index.co_je_noveho_subtitle")}</h3>
                <p>{t("home.index.co_je_noveho_text")}</p>
                <a class="btn-primary" href=path("/kontakt#cenik")>{t("common.cenik")}</a>
            </div>
        </section>
    }
}

/// "O nás": the family business.
#[component]
fn AboutUs() -> impl IntoView {
    view! {
        <section class="o-sobe-section">
            <div class="o-sobe-card">
                <div class="o-sobe-image-wrapper">
                    <img alt=t("home.index.o_sobe_image_alt") class="o-sobe-image" src=asset("o-sobe/nakladak.webp")/>
                </div>
                <div class="o-sobe-text">
                    <h2><Accented rest="home.index.o_sobe_title_rest" accent="home.index.o_sobe_title_accent"/></h2>
                    <p inner_html=t_html("home.index.o_sobe_p1_html")></p>
                    <p inner_html=t_html("home.index.o_sobe_p2_html")></p>
                    <p inner_html=t_html("home.index.o_sobe_p3_html")></p>
                    <p>{t("home.index.o_sobe_p4")}</p>
                    <p inner_html=t_html("home.index.o_sobe_p5_html")></p>
                    <a class="btn-primary" href=path("/kontakt#kalkulace")>{t("home.index.o_sobe_cta")}</a>
                </div>
            </div>
        </section>
    }
}
