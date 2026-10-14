//! GET /eshop/{key}: a product page. This file does what all three layouts
//! share (intro, title, breadcrumbs, recommendations); the layout's own part
//! comes from firewood.rs, lumber.rs or simple_variant.rs.

use axum::extract::{Path, State};
use minijinja::context;

use super::super::Result;
use super::super::components::format::strip_tags;
use super::super::not_found;
use super::variant_view::VariantViews;
use super::{firewood, lumber, recommendations, simple_variant};
use crate::AppState;
use crate::models::catalog::{self, CatalogProduct};
use crate::models::upload::{self, Upload};
use crate::request::Req;
use crate::views::{self, Crumb, Page};

pub async fn show(State(state): State<AppState>, req: Req, Path(key): Path<String>) -> Result {
    let Some(product) = catalog::active_product(&state.db, &key).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    let locale = req.locale;
    let t = |key: &str| state.translations.text(locale, key, &[]);
    let money = state.rates.money(locale).await;
    let variants = catalog::variants_of(&state.db, product.id).await?;
    let product_image = upload::images_of(&state.db, "CatalogProduct")
        .await?
        .get(&product.id)
        .map(Upload::url);
    let views = VariantViews {
        money,
        images: upload::images_of(&state.db, "CatalogVariant").await?,
    };
    let title = product.title(locale);

    let container = state.assets.url("eshop/container.webp");
    let layout = match product.template.as_str() {
        "firewood" => firewood::data(
            &variants,
            &views,
            &t("common.and"),
            &container,
            &state.assets.url("eshop/bedna.webp"),
        ),
        "lumber" => lumber::data(&variants, &views, product_image.clone()),
        _ => simple_variant::data(&variants, &views, &container),
    };

    let data = context! {
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
        recommendations => recommendations::for_product(&state, &product, &money).await?,
        ..layout
    };

    let mut page = Page::titled(
        &state,
        &req,
        &full_title(&product, &title, locale),
        Some(description(&state, &product, &title, locale)),
    );
    page.breadcrumbs = Some(breadcrumbs(&state, &req, &product, &title));
    Ok(views::render(
        &state,
        &req,
        &format!("eshop/{}.html", product.template),
        page,
        data,
    ))
}

/// "Smrk – Palivové dřevo"
fn full_title(product: &CatalogProduct, title: &str, locale: crate::locale::Locale) -> String {
    [title.to_string(), product.type_label(locale)]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" – ")
}

/// The description without HTML; a generic sentence when there is none.
fn description(state: &AppState, product: &CatalogProduct, title: &str, locale: crate::locale::Locale) -> String {
    let plain = strip_tags(&product.description(locale));
    if plain.trim().is_empty() {
        state
            .translations
            .text(locale, "meta.product_fallback", &[("title", title)])
    } else {
        plain
    }
}

/// E-shop > category > product (the category is left out when it would
/// repeat the product's name, e.g. Okrasné kamenivo).
fn breadcrumbs(state: &AppState, req: &Req, product: &CatalogProduct, title: &str) -> Vec<Crumb> {
    let t = |key: &str| state.translations.text(req.locale, key, &[]);
    let category = t(category_breadcrumb_key(&product.category));
    let mut crumbs = vec![Crumb {
        name: t("breadcrumbs.eshop"),
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

/// Category name for breadcrumbs, by product category.
pub fn category_breadcrumb_key(category: &str) -> &'static str {
    match category {
        "rezivo" => "breadcrumbs.stavebni_rezivo",
        "zbytky" => "breadcrumbs.vyrobni_zbytky",
        "kamenivo" => "breadcrumbs.okrasne_kamenivo",
        _ => "breadcrumbs.palivove_drevo",
    }
}
