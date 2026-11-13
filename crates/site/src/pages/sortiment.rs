//! The product range ("sortiment") and its five category pages.

#[cfg(feature = "ssr")]
use super::home::asset;
use super::home::{HowToOrder, Tile};
use super::{loaded, PageMeta};
use crate::i18n::t;
use crate::islands::SlideShow;
use crate::layout::Layout;
use crate::visitor::visitor;
use leptos::prelude::*;

#[server(endpoint = "page/sortiment")]
pub async fn sortiment_tiles() -> Result<Vec<Tile>, ServerFnError> {
    use crate::shop::translated as tr;
    let v = visitor();
    let l = v.locale;
    let db = &expect_context::<crate::state::AppState>().db;
    Ok(sqlx::query!("SELECT title, title_de, text, text_de, image, link FROM products ORDER BY id")
        .fetch_all(db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .into_iter()
        .map(|p| Tile {
            title: tr(l, &p.title, p.title_de.as_deref()).to_owned(),
            text: tr(l, p.text.as_deref().unwrap_or_default(), p.text_de.as_deref()).to_owned(),
            image: asset(p.image.as_deref().unwrap_or_default()),
            link: v.link(p.link.as_deref().unwrap_or("/")),
        })
        .collect())
}

#[component]
pub fn SortimentPage() -> impl IntoView {
    let l = visitor().locale;
    view! {
        <PageMeta title=t::meta::sortiment::title(l).to_owned() description=t::meta::sortiment::description(l)/>
        <Layout>
            <main class="main">
                <section class="sortiment-page-section">
                    <h2 class="section-main-title">{t::sortiment::index::title(l)}</h2>
                    {loaded(sortiment_tiles(), |tiles: Vec<Tile>| view! {
                        <div class="sortiment-grid-5">
                            {tiles.into_iter().enumerate().map(|(i, tile)| view! {
                                <a href=tile.link class="sortiment-card">
                                    <div class="sortiment-card-img">
                                        <img src=tile.image alt=tile.title.clone() loading=if i < 5 { "eager" } else { "lazy" }/>
                                    </div>
                                    <div class="sortiment-card-text">
                                        <h3>{tile.title}</h3>
                                        <p>{tile.text}</p>
                                    </div>
                                </a>
                            }).collect_view()}
                        </div>
                    })}
                    <HowToOrder/>
                </section>
            </main>
        </Layout>
    }
}

/// One product of a category page: text and buttons beside a slide show.
struct Row {
    title: &'static str,
    descriptions: Vec<&'static str>,
    images: &'static [&'static str],
    eshop_link: &'static str,
}

#[component]
fn ProductRow(row: Row, reverse: bool) -> impl IntoView {
    let v = visitor();
    let l = v.locale;
    use t::shared::product_row as p;
    view! {
        <section class=if reverse { "product-detail-hero reverse-layout" } else { "product-detail-hero" }>
            <div class="product-hero-text">
                <h2>{row.title}</h2>
                {row.descriptions.into_iter().map(|d| view! { <p>{d}</p> }).collect_view()}
                <div class="product-hero-buttons">
                    <a href=v.link("/kontakt#cenik") class="btn-primary">{p::cenik(l)}</a>
                    <a href=v.link(row.eshop_link) class="btn-outline-dark">{p::to_eshop(l)}</a>
                    <a href=v.link("/kontakt#kalkulace") class="btn-outline-dark">{p::request_quote(l)}</a>
                </div>
            </div>
            <SlideShow>
                <div class="sluzba-carousel-wrapper product-hero-carousel">
                    <button class="carousel-btn prev-btn" aria-label=t::common::previous(l)>
                        <svg viewBox="0 0 24 24" fill="white"><path d="M15.41 16.59L10.83 12l4.58-4.59L14 6l-6 6 6 6 1.41-1.41z"/></svg>
                    </button>
                    <div class="carousel-track-container">
                        <ul class="carousel-track">
                            {row.images.iter().enumerate().map(|(i, image)| view! {
                                <li class=if i == 0 { "carousel-slide active" } else { "carousel-slide" }>
                                    <img
                                        src=format!("/img/{image}")
                                        alt=row.title
                                        loading=if i == 0 { "eager" } else { "lazy" }
                                        fetchpriority=(i == 0).then_some("high")
                                    />
                                </li>
                            }).collect_view()}
                        </ul>
                    </div>
                    <button class="carousel-btn next-btn" aria-label=t::common::next(l)>
                        <svg viewBox="0 0 24 24" fill="white"><path d="M8.59 16.59L13.17 12 8.59 7.41 10 6l6 6-6 6-1.41-1.41z"/></svg>
                    </button>
                </div>
            </SlideShow>
        </section>
    }
}

#[component]
fn WoodTypes() -> impl IntoView {
    let l = visitor().locale;
    use t::shared::wood_types as w;
    let item = |img: &'static str, name: &'static str| view! {
        <div class="wood-item">
            <img src=format!("/img/sortiment/palivove-drevo/{img}.webp") class="wood-texture" alt=name loading="lazy"/>
            <span>{name}</span>
        </div>
    };
    view! {
        <section class="wood-types-section">
            <h2 class="wood-types-title" inner_html=w::title(l).into_string()></h2>
            <div class="wood-types-layout">
                <div class="wood-category-box">
                    <h3>{w::soft_title(l)}</h3>
                    <div class="wood-items-grid">
                        {item("smrk", w::soft_items::smrk(l))}
                        {item("borovice", w::soft_items::borovice(l))}
                        {item("modrin", w::soft_items::modrin(l))}
                    </div>
                </div>
                <div class="wood-category-box">
                    <h3>{w::hard_title(l)}</h3>
                    <div class="wood-items-grid">
                        {item("buk", w::hard_items::buk(l))}
                        {item("habr", w::hard_items::habr(l))}
                        {item("dub", w::hard_items::dub(l))}
                        {item("akat", w::hard_items::akat(l))}
                        {item("jasan", w::hard_items::jasan(l))}
                    </div>
                </div>
            </div>
        </section>
    }
}

/// A category page: its products, then (for wood) the wood types, then how to order.
fn category_page(title: &'static str, description: &'static str, rows: Vec<Row>, wood_types: bool, how_to_order_wrapper: bool) -> impl IntoView {
    view! {
        <PageMeta title=title.to_owned() description=description/>
        <Layout>
            <main class="main">
                {rows.into_iter().enumerate().map(|(i, row)| view! { <ProductRow row=row reverse=i % 2 == 1/> }).collect_view()}
                {wood_types.then(|| view! { <WoodTypes/> })}
                {if how_to_order_wrapper {
                    view! { <section class="how-to-order-detail-wrapper"><HowToOrder/></section> }.into_any()
                } else {
                    view! { <HowToOrder/> }.into_any()
                }}
            </main>
        </Layout>
    }
}

#[component]
pub fn PalivoveDrevoPage() -> impl IntoView {
    let l = visitor().locale;
    use t::sortiment::palivove_drevo as s;
    category_page(
        t::meta::palivove_drevo::title(l),
        t::meta::palivove_drevo::description(l),
        vec![Row {
            title: s::title(l),
            descriptions: vec![s::desc_1(l), s::desc_2(l)],
            images: &["sortiment/palivove-drevo1.webp", "sortiment/palivove-drevo2.webp"],
            eshop_link: "/eshop?category=palivove&",
        }],
        true,
        true,
    )
}

#[component]
pub fn StavebniRezivoPage() -> impl IntoView {
    let l = visitor().locale;
    use t::sortiment::stavebni_rezivo as s;
    category_page(
        t::meta::stavebni_rezivo::title(l),
        t::meta::stavebni_rezivo::description(l),
        vec![
            Row { title: s::tramy::title(l), descriptions: vec![s::tramy::desc_1(l), s::tramy::desc_2(l)], images: &["sortiment/tramy1.webp", "sortiment/tramy2.webp"], eshop_link: "/eshop/tramy" },
            Row { title: s::prkna::title(l), descriptions: vec![s::prkna::desc_1(l), s::prkna::desc_2(l)], images: &["sortiment/prkna1.webp", "sortiment/prkna2.webp"], eshop_link: "/eshop/prkna" },
            Row { title: s::late::title(l), descriptions: vec![s::late::desc_1(l), s::late::desc_2(l)], images: &["sortiment/late1.webp", "sortiment/late2.webp"], eshop_link: "/eshop/late" },
            Row { title: s::fosny::title(l), descriptions: vec![s::fosny::desc_1(l), s::fosny::desc_2(l)], images: &["sortiment/fosny1.webp", "sortiment/fosny2.webp"], eshop_link: "/eshop/fosny" },
        ],
        true,
        false,
    )
}

#[component]
pub fn TruhlarskeRezivoPage() -> impl IntoView {
    let l = visitor().locale;
    use t::sortiment::truhlarske_rezivo as s;
    category_page(
        t::meta::truhlarske_rezivo::title(l),
        t::meta::truhlarske_rezivo::description(l),
        vec![Row { title: s::fosny::title(l), descriptions: vec![s::fosny::desc_1(l)], images: &["sortiment/fosny3.webp", "sortiment/fosny1.webp"], eshop_link: "/eshop/fosny" }],
        true,
        false,
    )
}

#[component]
pub fn OkrasneKamenivoPage() -> impl IntoView {
    let l = visitor().locale;
    use t::sortiment::okrasne_kamenivo as s;
    category_page(
        t::meta::okrasne_kamenivo::title(l),
        t::meta::okrasne_kamenivo::description(l),
        vec![Row {
            title: s::title(l),
            descriptions: vec![s::desc_1(l), s::desc_2(l), s::desc_3(l), s::desc_4(l)],
            images: &["sortiment/kamenivo1.webp"],
            eshop_link: "/eshop/okrasne_kamenivo",
        }],
        false,
        false,
    )
}

#[component]
pub fn VyrobniZbytkyPage() -> impl IntoView {
    let l = visitor().locale;
    use t::sortiment::vyrobni_zbytky as s;
    category_page(
        t::meta::vyrobni_zbytky::title(l),
        t::meta::vyrobni_zbytky::description(l),
        vec![
            Row { title: s::odkory::title(l), descriptions: vec![s::odkory::desc_1(l)], images: &["sortiment/odrezky1.webp", "sortiment/odrezky2.webp"], eshop_link: "/eshop/odkory" },
            Row { title: s::stepka::title(l), descriptions: vec![s::stepka::desc_1(l)], images: &["sortiment/stepka1.webp", "sortiment/stepka2.webp"], eshop_link: "/eshop/stepka" },
            Row { title: s::piliny::title(l), descriptions: vec![s::piliny::desc_1(l)], images: &["sortiment/piliny1.webp", "sortiment/piliny2.webp"], eshop_link: "/eshop/piliny" },
        ],
        true,
        false,
    )
}
