//! The home page.

use super::{loaded, PageMeta, SafeHtml};
use crate::i18n::t;
use crate::islands::{ReviewsCarousel, SlideShow};
use crate::layout::Layout;
use crate::visitor::visitor;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HomeContent {
    promo: Option<Promo>,
    bestsellers: Vec<Bestseller>,
    products: Vec<Tile>,
    services: Vec<Service>,
    faq: Vec<Slide>,
    reviews: Vec<Review>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Promo {
    image: Option<String>,
    title: String,
    features: [String; 3],
    price: String,
    link: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bestseller {
    pub key: String,
    pub title: String,
    pub type_label: String,
    pub image: Option<String>,
    pub price_from: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub title: String,
    pub text: String,
    pub image: String,
    pub link: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Service {
    pub title: String,
    pub content: SafeHtml,
    pub images: Vec<String>,
    pub button: Option<(String, String)>,
    pub button_path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Slide {
    title: String,
    content: SafeHtml,
    image: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Review {
    name: String,
    initial: String,
    color: String,
    avatar: Option<String>,
    stars: usize,
    when: String,
    text: String,
}

/// A picture path from the database: the site's own pictures are stored
/// relative to /img ("sortiment/x.webp"), uploads as a full path.
pub fn asset(path: &str) -> String {
    if path.starts_with('/') { path.to_owned() } else { format!("/img/{path}") }
}

#[server(endpoint = "page/home")]
pub async fn home_data() -> Result<HomeContent, ServerFnError> {
    use crate::shop::translated as tr;
    let v = visitor();
    let l = v.locale;
    let db = &expect_context::<crate::state::AppState>().db;
    let err = |e: sqlx::Error| ServerFnError::new(e.to_string());

    let promos = sqlx::query!("SELECT * FROM promos ORDER BY id").fetch_all(db).await.map_err(err)?;
    let week = time::OffsetDateTime::now_utc().date().iso_week() as usize;
    let promo = (!promos.is_empty()).then(|| &promos[week % promos.len()]).map(|p| {
        let text = |cs: &Option<String>, de: &Option<String>| tr(l, cs.as_deref().unwrap_or_default(), de.as_deref()).to_owned();
        Promo {
            image: p.image.clone(),
            title: text(&p.title, &p.title_de),
            features: [text(&p.feature_1, &p.feature_1_de), text(&p.feature_2, &p.feature_2_de), text(&p.feature_3, &p.feature_3_de)],
            price: v.shop.price_text(l, &text(&p.price, &p.price_de)),
            link: v.link(p.link.as_deref().unwrap_or("/eshop")),
        }
    });

    let bestsellers = crate::catalog::bestsellers(db, &v, &["akat", "tramy", "dub"]).await.map_err(err)?.into_iter().take(3).collect();

    let products = sqlx::query!("SELECT title, title_de, text, text_de, image, link FROM products ORDER BY id")
        .fetch_all(db)
        .await
        .map_err(err)?
        .into_iter()
        .map(|p| Tile {
            title: tr(l, &p.title, p.title_de.as_deref()).to_owned(),
            text: tr(l, p.text.as_deref().unwrap_or_default(), p.text_de.as_deref()).to_owned(),
            image: asset(p.image.as_deref().unwrap_or_default()),
            link: v.link(p.link.as_deref().unwrap_or("/")),
        })
        .collect();

    let services = load_services(db, &v, None).await.map_err(err)?;

    let faq = sqlx::query!("SELECT title, title_de, content, content_de, image FROM faq_items ORDER BY id")
        .fetch_all(db)
        .await
        .map_err(err)?
        .into_iter()
        .map(|f| Slide {
            title: tr(l, f.title.as_deref().unwrap_or_default(), f.title_de.as_deref()).to_owned(),
            content: SafeHtml::from_admin(tr(l, f.content.as_deref().unwrap_or_default(), f.content_de.as_deref())),
            image: asset(f.image.as_deref().unwrap_or_default()),
        })
        .collect();

    const AVATAR_COLORS: [&str; 8] = ["#4285F4", "#EA4335", "#9C27B0", "#0F9D58", "#F4B400", "#FF7043", "#00ACC1", "#8E24AA"];
    let today = time::OffsetDateTime::now_utc().date();
    let reviews = sqlx::query!(r#"SELECT id AS "id!", name, text, stars, reviewed_on, avatar FROM reviews ORDER BY position, id"#)
        .fetch_all(db)
        .await
        .map_err(err)?
        .into_iter()
        .map(|r| Review {
            initial: r.name.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default(),
            color: AVATAR_COLORS[r.id.rem_euclid(AVATAR_COLORS.len() as i64) as usize].to_owned(),
            avatar: r.avatar.as_deref().filter(|a| !a.is_empty()).map(asset),
            stars: r.stars.clamp(1, 5) as usize,
            when: time::Date::parse(&r.reviewed_on, &time::format_description::well_known::Iso8601::DATE)
                .map(|d| time_ago(l, d, today))
                .unwrap_or_default(),
            name: r.name,
            text: r.text,
        })
        .collect();

    Ok(HomeContent { promo, bestsellers, products, services, faq, reviews })
}

/// The services (accordion on the home page, sections on the services
/// page), optionally only those whose button points somewhere under `prefix`.
#[cfg(feature = "ssr")]
pub async fn load_services(db: &sqlx::SqlitePool, v: &crate::visitor::Visitor, prefix: Option<&str>) -> Result<Vec<Service>, sqlx::Error> {
    use crate::shop::translated as tr;
    let l = v.locale;
    Ok(sqlx::query!("SELECT title, title_de, content, content_de, images, button_text, button_text_de, button_path FROM services ORDER BY id")
        .fetch_all(db)
        .await?
        .into_iter()
        .filter(|s| prefix.is_none_or(|p| s.button_path.as_deref().is_some_and(|b| b.starts_with(p))))
        .map(|s| Service {
            title: tr(l, s.title.as_deref().unwrap_or_default(), s.title_de.as_deref()).to_owned(),
            content: SafeHtml::from_admin(tr(l, s.content.as_deref().unwrap_or_default(), s.content_de.as_deref())),
            images: serde_json::from_str::<Vec<String>>(&s.images).unwrap_or_default().iter().map(|i| asset(i)).collect(),
            button: match (s.button_text.as_deref(), s.button_path.as_deref()) {
                (Some(text), Some(path)) if !text.is_empty() && !path.is_empty() => {
                    Some((tr(l, text, s.button_text_de.as_deref()).to_owned(), v.link(path)))
                }
                _ => None,
            },
            button_path: s.button_path,
        })
        .collect())
}

/// "před 2 lety", "vor 3 Monaten".
#[cfg(feature = "ssr")]
fn time_ago(l: crate::i18n::Locale, then: time::Date, today: time::Date) -> String {
    use crate::i18n::Locale;
    let mut years = today.year() - then.year();
    if (today.month() as u8, today.day()) < (then.month() as u8, then.day()) {
        years -= 1;
    }
    let (count, cs_one, cs_many, de_one, de_many) = if years >= 1 {
        (years, "rokem", "lety", "Jahr", "Jahren")
    } else {
        let mut months = (today.year() - then.year()) * 12 + i32::from(today.month() as u8) - i32::from(then.month() as u8);
        if today.day() < then.day() {
            months -= 1;
        }
        (months.max(1), "měsícem", "měsíci", "Monat", "Monaten")
    };
    match (l, count) {
        (Locale::Cs, 1) => format!("před {cs_one}"),
        (Locale::Cs, n) => format!("před {n} {cs_many}"),
        (Locale::De, 1) => format!("vor 1 {de_one}"),
        (Locale::De, n) => format!("vor {n} {de_many}"),
    }
}

/// "Jak objednat": online or by inquiry (home and e-shop pages).
#[component]
pub fn HowToOrder() -> impl IntoView {
    use t::shared::jak_objednat as j;
    let v = visitor();
    let l = v.locale;
    let steps = |list: &'static [&'static str]| {
        list.iter()
            .enumerate()
            .map(|(i, step)| view! { <li><strong>{i + 1}"."</strong>" "{*step}</li> })
            .collect_view()
    };
    view! {
        <h2 class="section-main-title jak-objednat-title" inner_html=j::title(l).into_string()></h2>
        <div class="jak-objednat-grid">
            <div class="jak-objednat-box">
                <h3>{j::online_title_rest(l)}" "<span class="text-blue">{j::online_title_accent(l)}</span></h3>
                <ul class="jak-objednat-steps">{steps(j::online_steps(l))}</ul>
                <img src="/img/jak-objednat/drevo.webp" class="jak-objednat-box-img" alt=j::online_img_alt(l) loading="lazy"/>
                <a href=v.link("/eshop") class="btn-primary full-width">{j::online_button(l)}</a>
            </div>
            <div class="jak-objednat-box">
                <h3>{j::poptavka_title_rest(l)}" "<span class="text-blue">{j::poptavka_title_accent(l)}</span></h3>
                <ul class="jak-objednat-steps">{steps(j::poptavka_steps(l))}</ul>
                <img src="/img/jak-objednat/pila.webp" class="jak-objednat-box-img" alt=j::poptavka_img_alt(l) loading="lazy"/>
                <a href=v.link("/kontakt#kalkulace") class="btn-primary full-width">{j::poptavka_button(l)}</a>
            </div>
        </div>
    }
}

/// An accordion item: the header opens and closes it (see PageBehaviour).
#[component]
pub fn AccordionItem(title: String, #[prop(optional)] faq: bool, children: Children) -> impl IntoView {
    view! {
        <div class=if faq { "accordion-item faq-item" } else { "accordion-item" }>
            <button class="accordion-header">
                <span class="accordion-title">{title}</span>
                {if faq {
                    view! { <span class="faq-icon"></span> }.into_any()
                } else {
                    view! {
                        <span class="accordion-icon">
                            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                        </span>
                    }.into_any()
                }}
            </button>
            <div class="accordion-content-wrapper">
                <div class="accordion-content">
                    <div class="accordion-body">{children()}</div>
                </div>
            </div>
        </div>
    }
}

/// A heading with its accented half in blue: "Nejprodávanější <span>produkty</span>".
fn accent(rest: &'static str, accent: &'static str) -> impl IntoView {
    view! { {rest}" "<span style="color: #2C8FFF">{accent}</span> }
}

#[component]
pub fn HomePage() -> impl IntoView {
    let v = visitor();
    let l = v.locale;
    use t::home::index as h;
    view! {
        <PageMeta title=t::meta::home::title(l).to_owned() description=t::meta::home::description(l) brand_only=true/>
        <crate::seo::LocalBusinessSchema/>
        <Layout>
            {loaded(home_data(), move |data: HomeContent| {
                let v = visitor();
                view! {
                    <main class="main">
                        <section class="hero-section">
                            <div class="hero-intro">
                                <div>
                                    <h1 class="fit-lines" data-fit-min="20"><span class="text-accent">{h::hero_title_accent(l)}</span><br/>{h::hero_title_rest(l)}</h1>
                                    <p>{h::hero_text(l)}</p>
                                </div>
                                <div class="domu-buttons">
                                    <a href=v.link("/eshop/") class="btn-eshop">{h::btn_eshop(l)}</a>
                                    <a href=v.link("/kontakt#kalkulace") class="btn-vyroba">{h::btn_vyroba(l)}</a>
                                </div>
                            </div>
                            {data.promo.map(|p| view! {
                                <div class="akcni-product-card">
                                    <div class="badge">{h::promo_badge(l)}</div>
                                    <div class="akcni-media">
                                        {p.image.map(|src| view! { <img src=src alt=h::promo_alt(l) class="hero-image" loading="eager" fetchpriority="high"/> })}
                                        <div class="product-info">
                                            <div class="product-subinfo">
                                                <h2 class="product-title">{p.title}</h2>
                                                <ul class="product-features">
                                                    {p.features.into_iter().map(|f| view! { <li>{f}</li> }).collect_view()}
                                                </ul>
                                            </div>
                                            <div class="akcni-product-price">
                                                <span class="akcni-nabidka-price">{p.price}</span>
                                                <a href=p.link class="akce-btn-eshop">{h::promo_cta(l)}</a>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            })}
                            <div class="hero-trust">
                                <div class="reviews">
                                    <img src="/img/google-review.webp" alt="Google recenze" class="google-logo" width="48" height="48"/>
                                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 370 48" width="280" height="48">
                                        <defs><g id="star"><path fill="#FABB05" d="M24 34.54l12.36 7.46-3.28-14.06 10.92-9.46-14.38-1.22L24 4 18.38 17.26 4 18.48l10.92 9.46-3.28 14.06z"/></g></defs>
                                        <use href="#star" x="0" y="0"/><use href="#star" x="60" y="0"/><use href="#star" x="120" y="0"/><use href="#star" x="180" y="0"/><use href="#star" x="240" y="0"/>
                                    </svg>
                                    <div class="review-text">
                                        <strong>"4.6 "</strong><span class="review-text-accent">{h::reviews_count(l)}</span>
                                    </div>
                                </div>
                                <div class="features">
                                    {[
                                        (h::features::doprava(l), "doprava.webp"),
                                        (h::features::vyroba(l), "vyroba.webp"),
                                        (h::features::sortiment(l), "sortiment.webp"),
                                        (h::features::vyrobce(l), "vyrobce.webp"),
                                        (h::features::garance(l), "garance.webp"),
                                        (h::features::ceny(l), "wallet.webp"),
                                    ].into_iter().map(|(name, img)| view! {
                                        <div class="feature-card">
                                            <div class="feature-item"><img src=format!("/img/features/{img}") alt=name.replace("<br>", " ") loading="lazy"/></div>
                                            // Our own translation, with a <br> in it.
                                            <span class="feature-text" inner_html=name></span>
                                        </div>
                                    }).collect_view()}
                                </div>
                            </div>
                            <div class="top-categories-section">
                                <h2 class="top-categories-title"><span class="brand-blue">{h::top_categories_title_accent(l)}</span>" "{h::top_categories_title_rest(l)}</h2>
                                <div class="top-categories-grid">
                                    {[
                                        (h::categories::palivove_drevo(l), "top-palivove-drevo.webp", "/sortiment/palivove-drevo"),
                                        (h::categories::stavebni_drevo(l), "top-stavebni-drevo.webp", "/sortiment/stavebni-rezivo"),
                                        (h::categories::vyrobni_zbytky(l), "top-vyrobni-zbytky.webp", "/sortiment/vyrobni-zbytky"),
                                    ].into_iter().map(|(name, img, path)| view! {
                                        <div class="top-categories-item">
                                            <a href=v.link(path) class="top-categories-image-link">
                                                <img src=format!("/img/top-kategorie/{img}") alt=name class="top-categories-image" loading="lazy"/>
                                            </a>
                                            <a href=v.link(path) class="top-categories-button">{name}</a>
                                        </div>
                                    }).collect_view()}
                                </div>
                            </div>
                        </section>
                        <section class="nejprodavanejsi-produkty-section">
                            <h1><span style="color: #2C8FFF">{h::nejprodavanejsi_title_accent(l)}</span>"    "{h::nejprodavanejsi_title_rest(l)}</h1>
                            <div class="nejprodavanejsi-produkty-grid">
                                {data.bestsellers.into_iter().map(|p| view! { <BestsellerCard p=p/> }).collect_view()}
                            </div>
                        </section>
                        <section class="akcni-nabidka-section">
                            <div class="akcni-nabidka-left">
                                <div class="akcni-nabidka-header">
                                    <h1>{accent(h::akcni_nabidka_title_rest(l), h::akcni_nabidka_title_accent(l))}</h1>
                                    <div class="akcni-nabidka">
                                        <h4 inner_html=h::akcni_nabidka_lead(l).into_string()></h4>
                                        <h4 inner_html=h::akcni_nabidka_text(l).into_string()></h4>
                                    </div>
                                    <div class="akcni-nabidka-cenik">
                                        <a href=v.link("/kontakt#cenik") class="akcni-nabidka-button">{t::common::cenik(l)}</a>
                                    </div>
                                </div>
                            </div>
                            <div class="akcni-nabidka-right">
                                <div class="akcni-nabidka-decoration">
                                    <div class="product-image-wrapper">
                                        <div class="akcni-badge">{h::akcni_badge(l)}</div>
                                        <img src="/img/akcni-nabidka.webp" alt=h::akcni_image_alt(l) class="akcni-nabidka-image" loading="lazy"/>
                                    </div>
                                </div>
                            </div>
                        </section>
                        <section class="nabidka-section">
                            <div class="nabidka-header">
                                <h2>{accent(h::nabidka_title_rest(l), h::nabidka_title_accent(l))}</h2>
                                <div class="nabidka-text">
                                    <p inner_html=h::nabidka_text_1(l).into_string()></p>
                                    <p inner_html=h::nabidka_text_2(l).into_string()></p>
                                </div>
                            </div>
                            <div class="nabidka-grid">
                                {data.products.into_iter().map(|item| view! {
                                    <div class="nabidka-card">
                                        <a href=item.link.clone() class="nabidka-image-link">
                                            <img src=item.image alt=item.title.clone() class="nabidka-image" loading="lazy"/>
                                        </a>
                                        <a href=item.link class="nabidka-btn">{item.title}</a>
                                    </div>
                                }).collect_view()}
                            </div>
                        </section>
                        <section class="sluzby-section">
                            <h2>{accent(h::sluzby_title_rest(l), h::sluzby_title_accent(l))}</h2>
                            <div class="accordion">
                                {data.services.into_iter().map(|s| view! {
                                    <AccordionItem title=s.title>
                                        <p inner_html=s.content.into_string()></p>
                                        {s.button.map(|(text, href)| view! { <a href=href class="btn-cenik">{text}</a> })}
                                    </AccordionItem>
                                }).collect_view()}
                            </div>
                        </section>
                        <section class="dorucujeme-section">
                            <div class="dorucujeme-text">
                                <h2>" "{accent(h::dorucujeme_title_rest(l), h::dorucujeme_title_accent(l))}</h2>
                                <p>{h::dorucujeme_text_1(l)}</p>
                                <p>{h::dorucujeme_text_2(l)}</p>
                                <a href=v.link("/kontakt#cenik") class="btn-primary">{t::common::cenik(l)}</a>
                            </div>
                            <div class="dorucujeme-map">
                                <div class="dorucujeme-map-inner">
                                    <img src="/img/mapa-doruceni.webp" alt=h::dorucujeme_map_alt(l) loading="lazy"/>
                                    <div class="dorucujeme-pin" tabindex="0" aria-label=format!("{}: {}", h::dorucujeme_pin_label(l), h::dorucujeme_pin_address(l))>
                                        <span class="dorucujeme-pin-tooltip">
                                            <strong>{h::dorucujeme_pin_label(l)}</strong>
                                            {h::dorucujeme_pin_address(l)}
                                        </span>
                                    </div>
                                </div>
                            </div>
                        </section>
                        <section class="co-je-noveho-section">
                            <div class="co-je-noveho-gallery">
                                <img src="/img/co-je-noveho-pic1.webp" alt=h::co_je_noveho_pic1_alt(l) class="co-je-noveho-img co-je-noveho-img-main" loading="lazy"/>
                                <img src="/img/co-je-noveho-pic2.webp" alt=h::co_je_noveho_pic2_alt(l) class="co-je-noveho-img" loading="lazy"/>
                                <img src="/img/co-je-noveho-pic3.webp" alt=h::co_je_noveho_pic3_alt(l) class="co-je-noveho-img" loading="lazy"/>
                            </div>
                            <div class="co-je-noveho-text">
                                <h2>{accent(h::co_je_noveho_title_rest(l), h::co_je_noveho_title_accent(l))}</h2>
                                <h3>{h::co_je_noveho_subtitle(l)}</h3>
                                <p>{h::co_je_noveho_text(l)}</p>
                                <a href=v.link("/kontakt#cenik") class="btn-primary">{t::common::cenik(l)}</a>
                            </div>
                        </section>
                        <section class="how-to-order-section">
                            <HowToOrder/>
                        </section>
                        <section class="co-by-vas-mohlo-zajimat">
                            <h2>{accent(h::zajimat_title_rest(l), h::zajimat_title_accent(l))}</h2>
                            <SlideShow>
                                <div class="carousel-wrapper" id="zajimatCarousel">
                                    <button class="carousel-btn prev-btn" id="carouselPrev" aria-label=t::common::previous(l)>
                                        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"></polyline></svg>
                                    </button>
                                    <div class="carousel-container">
                                        {data.faq.iter().cloned().enumerate().map(|(i, slide)| view! {
                                            <div class=if i == 0 { "carousel-slide active" } else { "carousel-slide" }>
                                                <img src=slide.image class="carousel-bg-img" alt=slide.title.clone() loading="lazy"/>
                                                <div class="carousel-overlay"></div>
                                                <div class="carousel-content">
                                                    <h3>{slide.title}</h3>
                                                    <div class="carousel-text"><p inner_html=slide.content.into_string()></p></div>
                                                </div>
                                            </div>
                                        }).collect_view()}
                                    </div>
                                    <button class="carousel-btn next-btn" id="carouselNext" aria-label=t::common::next(l)>
                                        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>
                                    </button>
                                </div>
                                <div class="carousel-dots">
                                    {(0..data.faq.len()).map(|i| view! {
                                        <button class=if i == 0 { "carousel-dot active" } else { "carousel-dot" } aria-label=format!("{} {}", t::common::slide(l), i + 1)></button>
                                    }).collect_view()}
                                </div>
                            </SlideShow>
                        </section>
                        <section class="faq-section">
                            <h2 class="faq-main-title">{accent(h::faq_title_rest(l), h::faq_title_accent(l))}</h2>
                            <FaqCategory title=h::faq_cat_palivove(l) items=vec![
                                (h::faq_palivove_q1(l), h::faq_palivove_a1(l)), (h::faq_palivove_q2(l), h::faq_palivove_a2(l)),
                                (h::faq_palivove_q3(l), h::faq_palivove_a3(l)), (h::faq_palivove_q4(l), h::faq_palivove_a4(l)),
                            ]/>
                            <FaqCategory title=h::faq_cat_stavebni(l) items=vec![
                                (h::faq_stavebni_q1(l), h::faq_stavebni_a1(l)), (h::faq_stavebni_q2(l), h::faq_stavebni_a2(l)),
                                (h::faq_stavebni_q3(l), h::faq_stavebni_a3(l)), (h::faq_stavebni_q4(l), h::faq_stavebni_a4(l)),
                            ]/>
                            <FaqCategory title=h::faq_cat_doprava(l) items=vec![
                                (h::faq_doprava_q1(l), h::faq_doprava_a1(l)), (h::faq_doprava_q2(l), h::faq_doprava_a2(l)),
                                (h::faq_doprava_q3(l), h::faq_doprava_a3(l)), (h::faq_doprava_q4(l), h::faq_doprava_a4(l)),
                            ]/>
                            <div class="faq-cta">
                                <div class="faq-cta-left">
                                    <span class="faq-cta-icon">
                                        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M20 15.5c-1.25 0-2.45-.2-3.57-.57a1.02 1.02 0 0 0-1.02.24l-2.2 2.2a15.045 15.045 0 0 1-6.59-6.59l2.2-2.21a.96.96 0 0 0 .25-1A11.36 11.36 0 0 1 8.5 4c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1 0 9.39 7.61 17 17 17 .55 0 1-.45 1-1v-3.5c0-.55-.45-1-1-1z"/></svg>
                                    </span>
                                    <span>{h::faq_cta_text(l)}</span>
                                </div>
                                <a href=v.link("/kontakt") class="btn-primary">{h::faq_cta_button(l)}</a>
                            </div>
                        </section>
                        <section class="o-sobe-section">
                            <div class="o-sobe-card">
                                <div class="o-sobe-image-wrapper">
                                    <img src="/img/o-sobe/nakladak.webp" alt=h::o_sobe_image_alt(l) class="o-sobe-image" loading="lazy"/>
                                </div>
                                <div class="o-sobe-text">
                                    <h2>{accent(h::o_sobe_title_rest(l), h::o_sobe_title_accent(l))}</h2>
                                    <p inner_html=h::o_sobe_p1(l).into_string()></p>
                                    <p inner_html=h::o_sobe_p2(l).into_string()></p>
                                    <p inner_html=h::o_sobe_p3(l).into_string()></p>
                                    <p>{h::o_sobe_p4(l)}</p>
                                    <p inner_html=h::o_sobe_p5(l).into_string()></p>
                                    <a href=v.link("/kontakt#kalkulace") class="btn-primary">{h::o_sobe_cta(l)}</a>
                                </div>
                            </div>
                        </section>
                        <section class="let-zkusenosti-section">
                            <div class="let-zkusenosti-banner"><h2>{h::zkusenosti_banner(l)}</h2></div>
                        </section>
                        <section class="google-reviews-section">
                            <h2>{accent(h::google_reviews_title_rest(l), h::google_reviews_title_accent(l))}</h2>
                            <div class="google-reviews-header">
                                <div class="google-reviews-rating">
                                    <img src="/img/google-review.webp" alt="Google" class="google-reviews-logo" loading="lazy"/>
                                    <div>
                                        <div class="google-reviews-score">"4.6 "<span class="google-reviews-stars">"★★★★★"</span></div>
                                        <div class="google-reviews-count">{h::google_reviews_count(l)}</div>
                                    </div>
                                </div>
                                <div class="google-reviews-divider"></div>
                                <div class="google-reviews-verified">
                                    <span class="google-reviews-check">
                                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><polyline points="8 12 11 15 16 9"></polyline></svg>
                                    </span>
                                    <div>
                                        <div class="google-reviews-verified-title">{h::google_reviews_verified_title(l)}</div>
                                        <div class="google-reviews-verified-sub">{h::google_reviews_verified_sub(l)}</div>
                                    </div>
                                </div>
                            </div>
                            <ReviewsCarousel previous=t::common::previous(l).to_owned() next=t::common::next(l).to_owned()>
                                {data.reviews.into_iter().map(|r| view! {
                                    <div class="google-review-card">
                                        <div class="google-review-card-header">
                                            {match r.avatar {
                                                Some(src) => view! { <img src=src alt=r.name.clone() class="google-review-avatar-img" loading="lazy"/> }.into_any(),
                                                None => view! { <div class="google-review-avatar" style=format!("background-color: {};", r.color)>{r.initial}</div> }.into_any(),
                                            }}
                                            <div>
                                                <div class="google-review-name">{r.name}</div>
                                                <div class="google-review-source">
                                                    <img src="/img/google-review.webp" alt="Google" class="google-review-source-icon" loading="lazy"/>
                                                    {h::google_review_source(l)}
                                                </div>
                                            </div>
                                        </div>
                                        <div class="google-review-meta">
                                            <span class="google-review-stars">{"★".repeat(r.stars)}</span>
                                            <span class="google-review-time">{r.when}</span>
                                        </div>
                                        <p class="google-review-text">{r.text}</p>
                                        <div class="google-review-like">
                                            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M1 21h4V9H1v12zm22-11c0-1.1-.9-2-2-2h-6.31l.95-4.57.03-.32c0-.41-.17-.79-.44-1.06L14.17 1 7.59 7.59C7.22 7.95 7 8.45 7 9v10c0 1.1.9 2 2 2h9c.83 0 1.54-.5 1.84-1.22l3.02-7.05c.09-.23.14-.47.14-.73v-2z"/></svg>
                                            {h::google_review_like(l)}
                                        </div>
                                    </div>
                                }).collect_view()}
                            </ReviewsCarousel>
                        </section>
                    </main>
                }
            })}
        </Layout>
    }
}

#[component]
fn FaqCategory(title: &'static str, items: Vec<(&'static str, &'static str)>) -> impl IntoView {
    view! {
        <h3 class="faq-category-title">{title}</h3>
        <div class="faq-grid">
            {items.into_iter().map(|(q, a)| view! {
                <AccordionItem title=q.to_owned() faq=true><p>{a}</p></AccordionItem>
            }).collect_view()}
        </div>
    }
}

#[component]
pub fn BestsellerCard(p: Bestseller) -> impl IntoView {
    let v = visitor();
    let l = v.locale;
    let href = v.link(&format!("/eshop/{}", p.key));
    view! {
        <div class="nejprodavanejsi-produkty-card">
            <a href=href.clone() class="nejprodavanejsi-produkty-image-link">
                {p.image.map(|src| view! { <img src=src alt=p.title.clone() class="nejprodavanejsi-produkty-image" loading="lazy"/> })}
            </a>
            <div class="nejprodavanejsi-produkty-details">
                <span class="nejprodavanejsi-produkty-title">{p.type_label}</span>
                <span class="nejprodavanejsi-produkty-desc"><strong>{p.title}</strong></span>
                <span class="nejprodavanejsi-produkty-stock">{t::common::stock(l)}</span>
                <span class="nejprodavanejsi-produkty-price">
                    <strong>{p.price_from.map_or_else(|| "---".to_owned(), |price| format!("{} {price}", t::common::price_from(l)))}</strong>
                </span>
                <a href=href class="btn-prohlednout">{t::home::index::btn_prohlednout(l)}</a>
            </div>
        </div>
    }
}
