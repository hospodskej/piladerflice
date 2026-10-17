//! Sortiment: the overview of tiles, and one page per kind of product
//! (firewood, building lumber...) with its products, photos and links.

use leptos::prelude::*;

use super::shared::{CategoryTile, HowToOrder, PhotoCarousel, WoodTypes};
use super::{asset, path, t};

#[component]
pub fn SortimentPage(tiles: Vec<CategoryTile>) -> impl IntoView {
    view! {
        <main class="main">
            <section class="sortiment-page-section">
                <h2 class="section-main-title">{t("sortiment.index.title")}</h2>
                <div class="sortiment-grid-5">
                    {tiles
                        .into_iter()
                        .map(|tile| {
                            view! {
                                <a class="sortiment-card" href=tile.link>
                                    <div class="sortiment-card-img">
                                        <img alt=tile.title.clone() src=tile.image.as_deref().map(asset)/>
                                    </div>
                                    <div class="sortiment-card-text">
                                        <h3>{tile.title}</h3>
                                        <p>{tile.text}</p>
                                    </div>
                                </a>
                            }
                        })
                        .collect_view()}
                </div>
                <HowToOrder/>
            </section>
        </main>
    }
}

/// A product on a Sortiment page: texts and buttons on one side, photos on
/// the other.
struct Product {
    title_key: &'static str,
    description_keys: &'static [&'static str],
    images: &'static [&'static str],
    eshop_link: &'static str,
}

/// The Sortiment pages, by their URL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Range {
    Firewood,
    BuildingLumber,
    JoineryLumber,
    Gravel,
    Offcuts,
}

impl Range {
    /// The `meta.*` translations of the page (title, description).
    pub fn meta_key(self) -> &'static str {
        match self {
            Range::Firewood => "palivove_drevo",
            Range::BuildingLumber => "stavebni_rezivo",
            Range::JoineryLumber => "truhlarske_rezivo",
            Range::Gravel => "okrasne_kamenivo",
            Range::Offcuts => "vyrobni_zbytky",
        }
    }

    fn products(self) -> &'static [Product] {
        match self {
            Range::Firewood => &[Product {
                title_key: "sortiment.palivove_drevo.title",
                description_keys: &["sortiment.palivove_drevo.desc_1", "sortiment.palivove_drevo.desc_2"],
                images: &["sortiment/palivove-drevo1.webp", "sortiment/palivove-drevo2.webp"],
                eshop_link: "/eshop?category=palivove&",
            }],
            Range::BuildingLumber => &[
                Product {
                    title_key: "sortiment.stavebni_rezivo.tramy.title",
                    description_keys: &[
                        "sortiment.stavebni_rezivo.tramy.desc_1",
                        "sortiment.stavebni_rezivo.tramy.desc_2",
                    ],
                    images: &["sortiment/tramy1.webp", "sortiment/tramy2.webp"],
                    eshop_link: "/eshop/tramy",
                },
                Product {
                    title_key: "sortiment.stavebni_rezivo.prkna.title",
                    description_keys: &[
                        "sortiment.stavebni_rezivo.prkna.desc_1",
                        "sortiment.stavebni_rezivo.prkna.desc_2",
                    ],
                    images: &["sortiment/prkna1.webp", "sortiment/prkna2.webp"],
                    eshop_link: "/eshop/prkna",
                },
                Product {
                    title_key: "sortiment.stavebni_rezivo.late.title",
                    description_keys: &[
                        "sortiment.stavebni_rezivo.late.desc_1",
                        "sortiment.stavebni_rezivo.late.desc_2",
                    ],
                    images: &["sortiment/late1.webp", "sortiment/late2.webp"],
                    eshop_link: "/eshop/late",
                },
                Product {
                    title_key: "sortiment.stavebni_rezivo.fosny.title",
                    description_keys: &[
                        "sortiment.stavebni_rezivo.fosny.desc_1",
                        "sortiment.stavebni_rezivo.fosny.desc_2",
                    ],
                    images: &["sortiment/fosny1.webp", "sortiment/fosny2.webp"],
                    eshop_link: "/eshop/fosny",
                },
            ],
            Range::JoineryLumber => &[Product {
                title_key: "sortiment.truhlarske_rezivo.fosny.title",
                description_keys: &["sortiment.truhlarske_rezivo.fosny.desc_1"],
                images: &["sortiment/fosny3.webp", "sortiment/fosny1.webp"],
                eshop_link: "/eshop/fosny",
            }],
            Range::Gravel => &[Product {
                title_key: "sortiment.okrasne_kamenivo.title",
                description_keys: &[
                    "sortiment.okrasne_kamenivo.desc_1",
                    "sortiment.okrasne_kamenivo.desc_2",
                    "sortiment.okrasne_kamenivo.desc_3",
                    "sortiment.okrasne_kamenivo.desc_4",
                ],
                images: &["sortiment/kamenivo1.webp"],
                eshop_link: "/eshop/okrasne_kamenivo",
            }],
            Range::Offcuts => &[
                Product {
                    title_key: "sortiment.vyrobni_zbytky.odkory.title",
                    description_keys: &["sortiment.vyrobni_zbytky.odkory.desc_1"],
                    images: &["sortiment/odrezky1.webp", "sortiment/odrezky2.webp"],
                    eshop_link: "/eshop/odkory",
                },
                Product {
                    title_key: "sortiment.vyrobni_zbytky.stepka.title",
                    description_keys: &["sortiment.vyrobni_zbytky.stepka.desc_1"],
                    images: &["sortiment/stepka1.webp", "sortiment/stepka2.webp"],
                    eshop_link: "/eshop/stepka",
                },
                Product {
                    title_key: "sortiment.vyrobni_zbytky.piliny.title",
                    description_keys: &["sortiment.vyrobni_zbytky.piliny.desc_1"],
                    images: &["sortiment/piliny1.webp", "sortiment/piliny2.webp"],
                    eshop_link: "/eshop/piliny",
                },
            ],
        }
    }
}

#[component]
pub fn RangePage(range: Range) -> impl IntoView {
    let how_to_order = if range == Range::Firewood {
        view! {
            <section class="how-to-order-detail-wrapper">
                <HowToOrder/>
            </section>
        }
        .into_any()
    } else {
        view! { <HowToOrder/> }.into_any()
    };
    view! {
        <main class="main">
            {range
                .products()
                .iter()
                .enumerate()
                .map(|(index, product)| view! { <ProductRow product=product reverse=index % 2 == 1/> })
                .collect_view()}
            {(range != Range::Gravel).then(|| view! { <WoodTypes/> })}
            {how_to_order}
        </main>
    }
}

/// One product, alternating sides (`reverse`).
#[component]
fn ProductRow(product: &'static Product, reverse: bool) -> impl IntoView {
    let title = t(product.title_key);
    view! {
        <section class="product-detail-hero" class:reverse-layout=reverse>
            <div class="product-hero-text">
                <h2>{title.clone()}</h2>
                {product.description_keys.iter().map(|key| view! { <p>{t(key)}</p> }).collect_view()}
                <div class="product-hero-buttons">
                    <a class="btn-primary" href=path("/kontakt#cenik")>{t("shared.product_row.cenik")}</a>
                    " "
                    <a class="btn-outline-dark" href=path(product.eshop_link)>{t("shared.product_row.to_eshop")}</a>
                    " "
                    <a class="btn-outline-dark" href=path("/kontakt#kalkulace")>{t("shared.product_row.request_quote")}</a>
                </div>
            </div>
            <PhotoCarousel
                images=product.images.iter().map(|image| asset(image)).collect()
                alt=title
                class="product-hero-carousel"
            />
        </section>
    }
}
