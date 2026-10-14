//! The e-shop: the product grid with its filters, and the product pages
//! (three layouts: firewood, lumber, and simple variants).

use axum::extract::{Path, State};
use axum::response::IntoResponse;
use minijinja::context;
use rand::seq::SliceRandom;
use serde::Serialize;

use super::Result;
use super::catalog::{ProductCard, category_breadcrumb_key};
use crate::AppState;
use crate::models::catalog::{self, CATEGORIES, CatalogProduct, CatalogVariant, FirewoodGrid};
use crate::models::uploads::{self, Upload};
use crate::money::Money;
use crate::request::Req;
use crate::views::{self, Crumb, Page};

fn t(state: &AppState, req: &Req, key: &str) -> String {
    state.translations.text(req.locale, key, &[])
}

/// GET /eshop?category=&wood=&sort=
pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let param = |name: &str| req.query_param(name).unwrap_or_default();
    let (category_param, wood, sort) = (param("category"), param("wood"), param("sort"));
    let category = if CATEGORIES.contains(&category_param.as_str()) {
        category_param.as_str()
    } else {
        "palivove"
    };

    let money = state.rates.money(req.locale).await;
    let variants = catalog::all_variants(&state.db).await?;
    let images = uploads::images_of(&state.db, "CatalogProduct").await?;

    let mut products: Vec<CatalogProduct> = catalog::active_products(&state.db)
        .await?
        .into_iter()
        .filter(|product| product.category == category)
        .filter(|product| match wood.as_str() {
            "tvrde" => product.hardness.as_deref() == Some("hard"),
            "mekke" => product.hardness.as_deref() == Some("soft"),
            _ => true,
        })
        .collect();
    let order: &[&str] = if sort == "nejprodavanejsi" {
        &[
            "buk",
            "dub",
            "smrk",
            "tramy",
            "fosny",
            "late",
            "piliny",
            "odkory",
            "stepka",
            "okrasne_kamenivo",
        ]
    } else {
        &[
            "smrk",
            "dub",
            "buk",
            "late",
            "tramy",
            "fosny",
            "stepka",
            "odkory",
            "piliny",
            "okrasne_kamenivo",
        ]
    };
    // A stable sort, like Ruby's sort_by on these keys.
    products.sort_by_key(|product| order.iter().position(|key| *key == product.key).unwrap_or(999));

    let price_from = t(&state, &req, "common.price_from");
    let cards: Vec<ProductCard> = products
        .iter()
        .map(|product| ProductCard::new(product, &variants, &images, &money, &price_from))
        .collect();

    let page = Page::meta(&state, &req, &format!("eshop_{category}"));
    Ok(views::render(
        &state,
        &req,
        "home/eshop.html",
        page,
        context! { cards, category_param, wood, sort, palivove => category_param.is_empty() || category_param == "palivove" },
    ))
}

#[derive(Serialize)]
struct VariantView {
    id: i64,
    in_stock: bool,
    price: String,
    image: Option<String>,
    stacked: bool,
    length: String,
    amount: Option<i64>,
    grade: Option<String>,
    width_mm: Option<i64>,
    height_mm: Option<i64>,
    label: String,
    filter_value: String,
}

#[derive(Serialize)]
struct GridRow {
    label: String,
    cells: Vec<Option<VariantView>>,
}

#[derive(Serialize)]
struct FilterOption {
    key: String,
    label: String,
}

/// GET /eshop/{key}
pub async fn product(State(state): State<AppState>, req: Req, Path(key): Path<String>) -> Result {
    let Some(product) = catalog::active_product(&state.db, &key).await? else {
        return Ok(super::errors::not_found(State(state), req).await);
    };
    let locale = req.locale;
    let money = state.rates.money(locale).await;
    let variants = catalog::variants_of(&state.db, product.id).await?;
    let product_images = uploads::images_of(&state.db, "CatalogProduct").await?;
    let variant_images = uploads::images_of(&state.db, "CatalogVariant").await?;
    let product_image = product_images.get(&product.id).map(Upload::url);
    let title = product.title(locale);

    let view = |variant: &CatalogVariant, fallback: Option<String>, filter_value: String| VariantView {
        id: variant.id,
        in_stock: variant.in_stock,
        price: money.display(variant.price_czk),
        image: variant_images.get(&variant.id).map(Upload::url).or(fallback),
        stacked: variant.is_stacked(),
        length: variant.length_label.clone().unwrap_or_default(),
        amount: variant.amount_value,
        grade: variant
            .grade
            .as_deref()
            .filter(|g| !g.trim().is_empty())
            .map(|_| variant.grade(locale)),
        width_mm: variant.width_mm,
        height_mm: variant.height_mm,
        label: variant.variant_label(locale),
        filter_value,
    };
    let asset = |logical: &str| Some(state.assets.url(logical));

    let mut data = context! {
        product => context! {
            key => &product.key,
            title => &title,
            subtitle => product.subtitle(locale),
            description => product.description(locale),
            drying_note => product.drying_note(locale),
            image_alt => product.image_alt(locale),
            type_label => product.type_label(locale),
            image => &product_image,
        },
        template => &product.template,
        recommendations => recommendations(&state, &req, &product, &money).await?,
    };

    match product.template.as_str() {
        "firewood" => {
            let kontejner: Vec<&CatalogVariant> = variants
                .iter()
                .filter(|v| v.variant_group.as_deref() == Some("kontejner"))
                .collect();
            let grid = FirewoodGrid::new(kontejner);
            let lengths = grid.lengths();
            let and = format!(" {} ", t(&state, &req, "common.and"));
            let rows: Vec<GridRow> = grid
                .rows()
                .iter()
                .enumerate()
                .map(|(index, amounts)| GridRow {
                    label: amounts.iter().map(i64::to_string).collect::<Vec<_>>().join(&and),
                    cells: lengths
                        .iter()
                        .map(|length| {
                            grid.variant_at(index, length)
                                .map(|v| view(v, asset("eshop/container.webp"), String::new()))
                        })
                        .collect(),
                })
                .collect();
            let bedny: Vec<VariantView> = variants
                .iter()
                .filter(|v| v.variant_group.as_deref() == Some("bedny"))
                .map(|v| view(v, asset("eshop/bedna.webp"), String::new()))
                .collect();
            data = context! { lengths, rows, bedny, ..data };
        }
        "lumber" => {
            let has_dimensions = variants.iter().any(|v| v.dimension_filter_key().is_some());
            let filter_key = |v: &CatalogVariant| {
                if has_dimensions {
                    v.dimension_filter_key()
                } else {
                    v.grade_filter_key()
                }
            };
            let mut filters: Vec<FilterOption> = Vec::new();
            for variant in &variants {
                let Some(key) = filter_key(variant).filter(|key| !key.is_empty()) else {
                    continue;
                };
                if filters.iter().any(|option| option.key == key) {
                    continue;
                }
                let label = if has_dimensions {
                    format!(
                        "{}×{} mm",
                        variant.width_mm.unwrap_or_default(),
                        variant.height_mm.unwrap_or_default()
                    )
                } else {
                    variant.grade(locale)
                };
                filters.push(FilterOption { key, label });
            }
            let cards: Vec<VariantView> = variants
                .iter()
                .map(|v| view(v, product_image.clone(), filter_key(v).unwrap_or_default()))
                .collect();
            data = context! { has_dimensions, filters, cards, ..data };
        }
        _ => {
            let cards: Vec<VariantView> = variants
                .iter()
                .map(|v| view(v, asset("eshop/container.webp"), String::new()))
                .collect();
            let grid_class = if variants.len() <= 2 {
                "simple-grid-2"
            } else {
                "simple-grid-3"
            };
            data = context! { cards, grid_class, ..data };
        }
    }

    let full_title = [title.clone(), product.type_label(locale)]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" – ");
    let plain_description = strip_tags(&product.description(locale));
    let description = if plain_description.trim().is_empty() {
        state
            .translations
            .text(locale, "meta.product_fallback", &[("title", &title)])
    } else {
        plain_description
    };
    let mut page = Page::titled(&state, &req, &full_title, Some(description));
    page.breadcrumbs = Some(product_breadcrumbs(&state, &req, &product, &title));

    Ok(views::render(&state, &req, &format!("eshop/{}.html", product.template), page, data).into_response())
}

/// E-shop > category > product (the category is left out when it would
/// repeat the product's name, e.g. Okrasné kamenivo).
fn product_breadcrumbs(state: &AppState, req: &Req, product: &CatalogProduct, title: &str) -> Vec<Crumb> {
    let category = t(state, req, category_breadcrumb_key(&product.category));
    let mut crumbs = vec![Crumb {
        name: t(state, req, "breadcrumbs.eshop"),
        href: req.path_for("/eshop"),
    }];
    if category != title {
        crumbs.push(Crumb {
            name: category,
            href: req.path_for(&format!("/eshop?category={}", product.category)),
        });
    }
    crumbs.push(Crumb {
        name: title.to_string(),
        href: "#".into(),
    });
    crumbs
}

/// Four other products, picked at random.
async fn recommendations(
    state: &AppState,
    req: &Req,
    current: &CatalogProduct,
    money: &Money,
) -> Result<Vec<ProductCard>> {
    let mut others: Vec<CatalogProduct> = catalog::active_products(&state.db)
        .await?
        .into_iter()
        .filter(|p| p.key != current.key)
        .collect();
    others.shuffle(&mut rand::rng());
    others.truncate(4);
    let variants = catalog::all_variants(&state.db).await?;
    let images = uploads::images_of(&state.db, "CatalogProduct").await?;
    let price_from = t(state, req, "common.price_from");
    Ok(others
        .iter()
        .map(|product| ProductCard::new(product, &variants, &images, money, &price_from))
        .collect())
}

/// The text of an HTML fragment, without its tags (Rails' `strip_tags`).
fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_tags() {
        assert_eq!(strip_tags("Suché <strong>dřevo</strong><br>"), "Suché dřevo");
    }
}
