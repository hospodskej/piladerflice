//! The e-shop: the product grid with its filters, and the product pages. A
//! product page has one of three layouts (firewood, lumber, simple), each
//! in its own file.

mod buy;
mod firewood;
mod listing;
mod lumber;
mod simple;
mod variant;

pub use firewood::{FirewoodRow, FirewoodVariants};
pub use listing::{Listing, ListingPage};
pub use lumber::{FilterOption, LumberVariants};
pub use simple::SimpleVariants;
pub use variant::{VariantView, VariantViews};

use leptos::prelude::*;

use super::shared::{CustomDemand, ProductCard, ProductCardView};
use super::{t, t_html};

/// What every product page shows about the product itself.
pub struct ProductInfo {
    pub key: String,
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub drying_note: String,
    pub image_alt: String,
    pub type_label: String,
    pub image: Option<String>,
}

/// The variants, laid out the product's way.
pub enum ProductLayout {
    Firewood(FirewoodVariants),
    Lumber(LumberVariants),
    Simple(SimpleVariants),
}

impl ProductLayout {
    fn kind(&self) -> Kind {
        match self {
            ProductLayout::Firewood(_) => Kind::Firewood,
            ProductLayout::Lumber(_) => Kind::Lumber,
            ProductLayout::Simple(_) => Kind::Simple,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Firewood,
    Lumber,
    Simple,
}

#[component]
pub fn ProductPage(product: ProductInfo, layout: ProductLayout, recommendations: Vec<ProductCard>) -> impl IntoView {
    let kind = layout.kind();
    let variants = match layout {
        ProductLayout::Firewood(variants) => view! { <firewood::Firewood variants=variants/> }.into_any(),
        ProductLayout::Lumber(variants) => {
            view! { <lumber::Lumber variants=variants title=product.title.clone()/> }.into_any()
        }
        ProductLayout::Simple(variants) => {
            view! { <simple::Simple variants=variants type_label=product.type_label.clone()/> }.into_any()
        }
    };
    view! {
        <main class="product-page-main main">
            <div class="product-container">
                <Intro product=product kind=kind/>
                {variants}
                <Recommendations products=recommendations/>
            </div>
            <CustomDemand/>
        </main>
    }
}

/// Picture, name and description at the top of a product page.
#[component]
fn Intro(product: ProductInfo, kind: Kind) -> impl IntoView {
    let firewood = kind == Kind::Firewood;
    let alt = if firewood {
        product.image_alt.clone()
    } else {
        product.title.clone()
    };
    view! {
        <section class="product-intro-section">
            <div class="product-intro-image">{product.image.map(|src| view! { <img alt=alt src=src/> })}</div>
            <div class="product-intro-details">
                <h1>{product.title}</h1>
                {firewood.then(|| view! { <p class="subtitle">{product.subtitle.clone()}</p> })}
                <p class="description">{product.description}</p>
                {if firewood {
                    view! {
                        <p class="drying-note">{product.drying_note}</p>
                        <p class="stock-status">{t("eshop.firewood.stock_status")}</p>
                    }
                        .into_any()
                } else if kind == Kind::Lumber {
                    view! { <p class="stock-status">{t("common.in_stock")}</p> }.into_any()
                } else {
                    ().into_any()
                }}
            </div>
        </section>
    }
}

/// "You might also like": four other products.
#[component]
fn Recommendations(products: Vec<ProductCard>) -> impl IntoView {
    view! {
        <section class="recommendations-section">
            <h2 class="section-title text-center" inner_html=t_html("shared.recommendations.title_html")></h2>
            <div class="eshop-grid">
                {products.into_iter().map(|product| view! { <ProductCardView product=product/> }).collect_view()}
            </div>
        </section>
    }
}
