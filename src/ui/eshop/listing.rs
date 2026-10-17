//! The product grid, with filters by category and kind of wood, and two
//! orders.

use leptos::prelude::*;

use crate::ui::shared::{CustomDemand, ProductCard, ProductCardView};
use crate::ui::{path, t};

/// The grid and the filters as chosen in the URL.
pub struct Listing {
    pub cards: Vec<ProductCard>,
    /// As given (may be empty).
    pub category: String,
    pub wood: String,
    pub sort: String,
    /// The firewood filters show.
    pub firewood: bool,
}

#[component]
pub fn ListingPage(listing: Listing) -> impl IntoView {
    let Listing {
        cards,
        category,
        wood,
        sort,
        firewood,
    } = listing;
    let category_link = |key: &str, label: &str, active: bool| {
        view! {
            <a class="btn-filter" class:active=active href=path(&format!("/eshop?category={key}&sort={sort}"))>{t(label)}</a>
        }
    };
    let wood_link = |key: &str, label: &str, active: bool| {
        view! {
            <a class="btn-filter btn-filter-sm" class:active=active href=path(&format!("/eshop?category={category}&wood={key}&sort={sort}"))>
                {t(label)}
            </a>
        }
    };
    let sort_link = |key: &str, label: &str, active: bool| {
        view! { <a class="btn-sort" class:active=active href=path(&format!("/eshop?category={category}&wood={wood}&sort={key}"))>{t(label)}</a> }
    };
    view! {
        <main class="eshop-main main">
            <turbo-frame id="eshop_content" data-turbo-action="advance">
                <div class="eshop-container">
                    <section class="eshop-filters">
                        <h3>{t("eshop.index.filters_title")}</h3>
                        <div class="filter-group">
                            {category_link("palivove", "eshop.index.cat_palivove", firewood)}
                            " "
                            {category_link("rezivo", "eshop.index.cat_rezivo", category == "rezivo")}
                            " "
                            {category_link("zbytky", "eshop.index.cat_zbytky", category == "zbytky")}
                            " "
                            {category_link("kamenivo", "eshop.index.cat_kamenivo", category == "kamenivo")}
                        </div>
                        {firewood
                            .then(|| {
                                view! {
                                    <h3 class="mt-40">{t("eshop.index.wood_type_title")}</h3>
                                    <div class="filter-group">
                                        {wood_link("vse", "eshop.index.wood_all", wood.is_empty() || wood == "vse")}
                                        " "
                                        {wood_link("tvrde", "eshop.index.wood_hard", wood == "tvrde")}
                                        " "
                                        {wood_link("mekke", "eshop.index.wood_soft", wood == "mekke")}
                                    </div>
                                }
                            })}
                    </section>
                    <section class="eshop-sorting">
                        <div class="sorting-tabs">
                            {sort_link("doporucujeme", "eshop.index.sort_recommended", sort.is_empty() || sort == "doporucujeme")}
                            " "
                            {sort_link("nejprodavanejsi", "eshop.index.sort_bestsellers", sort == "nejprodavanejsi")}
                        </div>
                        <hr class="sorting-divider"/>
                    </section>
                    <section class="eshop-grid">
                        {cards
                            .into_iter()
                            .enumerate()
                            .map(|(index, product)| {
                                let badge = (index < 3).then_some(index + 1);
                                view! { <ProductCardView product=product badge=badge/> }
                            })
                            .collect_view()}
                    </section>
                </div>
            </turbo-frame>
            <CustomDemand/>
        </main>
    }
}
