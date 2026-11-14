//! The total rows under a cart.

use crate::cart_view::{RowKind, TotalRow};
use leptos::prelude::*;

#[component]
pub fn TotalRows(rows: Vec<TotalRow>) -> impl IntoView {
    rows.into_iter()
        .map(|row| match row.kind {
            RowKind::Total => view! {
                <div class="cart-total-row"><span>{row.label}</span><strong>{row.value}</strong></div>
            }
            .into_any(),
            kind => {
                let class = match kind {
                    RowKind::Discount => "cart-total-row cart-total-discount",
                    RowKind::Delivery => "cart-total-row cart-total-excl-vat cart-total-delivery",
                    _ => "cart-total-row cart-total-excl-vat",
                };
                view! { <div class=class><span>{row.label}</span><span>{row.value}</span></div> }.into_any()
            }
        })
        .collect_view()
}
