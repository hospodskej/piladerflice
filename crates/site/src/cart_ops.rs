//! Changing the cart: adding a variant (with its price and description as
//! they are now), quantities, removing, discount codes, delivery estimates.

use crate::session::{SessionError, save_cart};
use crate::state::AppState;
use crate::visitor::Visitor;
use domain::Category;
use domain::cart::{CartLine, Spec};
use domain::catalog::Template;
use domain::money::Czk;
use tower_sessions::Session;

#[derive(Debug, thiserror::Error)]
pub enum CartError {
    #[error("no such variant for sale")]
    NotForSale,
    #[error(transparent)]
    Session(#[from] SessionError),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// A variant's description in the cart, by its product's layout.
fn specs(
    template: Template,
    stacked: bool,
    length: Option<String>,
    amount: Option<i64>,
    grade: Option<(String, Option<String>)>,
    size: Option<(i64, i64)>,
    label: (Option<String>, Option<String>),
) -> Vec<Spec> {
    let mut specs = Vec::new();
    match template {
        Template::Firewood => {
            let key = if stacked {
                "common.stacked_variant"
            } else {
                "common.loose_variant"
            };
            specs.push(Spec::Key { value: key.into() });
            specs.extend(
                length
                    .filter(|l| !l.is_empty())
                    .map(|value| Spec::Raw { value }),
            );
            specs.extend(amount.map(|value| Spec::Amount {
                value,
                unit_key: "common.per_m3".into(),
            }));
        }
        Template::Lumber => {
            specs.extend(
                grade
                    .filter(|(g, _)| !g.is_empty())
                    .map(|(value, value_de)| Spec::Bilingual { value, value_de }),
            );
            specs.extend(
                length
                    .filter(|l| !l.is_empty())
                    .map(|value| Spec::Raw { value }),
            );
            specs.extend(size.map(|(w, h)| Spec::Raw {
                value: format!("{w}×{h} mm"),
            }));
        }
        Template::SimpleVariant => {
            specs.push(Spec::Bilingual {
                value: label.0.unwrap_or_default(),
                value_de: label.1,
            });
        }
    }
    specs
}

/// Adds `quantity` of a variant on sale (its product is active) and saves the cart.
pub async fn add(
    state: &AppState,
    session: &Session,
    v: &mut Visitor,
    variant_id: i64,
    quantity: u32,
) -> Result<(), CartError> {
    let row = sqlx::query!(
        r#"SELECT v.price_czk, v.stacked, v.variant_group, v.length_label, v.amount_value, v.grade, v.grade_de,
                  v.width_mm, v.height_mm, v.variant_label, v.variant_label_de, v.image AS variant_image,
                  p.key, p.template, p.category, p.image AS product_image
           FROM catalog_variants v JOIN catalog_products p ON p.id = v.catalog_product_id
           WHERE v.id = ? AND p.active = 1"#,
        variant_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(CartError::NotForSale)?;
    let category: Category = row.category.parse().map_err(|_| CartError::NotForSale)?;
    let template: Template = row.template.parse().map_err(|_| CartError::NotForSale)?;
    let stacked = row.stacked != 0 || row.variant_group.as_deref() == Some("bedny");
    let line = CartLine {
        variant_id,
        product_key: row.key,
        image: row.variant_image.or(row.product_image),
        category,
        unit_price: v.shop.vat.customer_price(category, Czk(row.price_czk)),
        specs: specs(
            template,
            stacked,
            row.length_label,
            row.amount_value,
            row.grade.map(|g| (g, row.grade_de)),
            row.width_mm.zip(row.height_mm),
            (row.variant_label, row.variant_label_de),
        ),
        quantity,
    };
    v.cart.add(line);
    save_cart(session, &v.cart).await?;
    Ok(())
}

pub async fn set_quantity(
    session: &Session,
    v: &mut Visitor,
    variant_id: i64,
    quantity: u32,
) -> Result<(), CartError> {
    v.cart.set_quantity(variant_id, quantity);
    Ok(save_cart(session, &v.cart).await?)
}

/// Why a discount code wasn't taken, or `None` when it was.
pub async fn apply_discount(
    state: &AppState,
    session: &Session,
    v: &mut Visitor,
    code: &str,
) -> Result<Option<(domain::discount::Unusable, Option<Czk>)>, CartError> {
    use domain::discount::Unusable;
    let found = match domain::discount::normalize_code(code) {
        Some(code) => crate::session::discount_code(&state.db, &code).await?,
        None => None,
    };
    let Some(found) = found else {
        return Ok(Some((Unusable::NotFound, None)));
    };
    if let Some(reason) =
        found.unusable_reason(v.cart.subtotal(), time::OffsetDateTime::now_utc().date())
    {
        return Ok(Some((reason, found.min_order)));
    }
    v.cart.discount_code = Some(found.code.clone());
    v.discount = Some(found);
    save_cart(session, &v.cart).await?;
    Ok(None)
}

pub async fn remove_discount(session: &Session, v: &mut Visitor) -> Result<(), CartError> {
    v.cart.discount_code = None;
    v.discount = None;
    Ok(save_cart(session, &v.cart).await?)
}
