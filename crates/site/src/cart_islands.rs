//! The cart in the header and the add-to-cart buttons. Adding goes through
//! a server function; the header's cart follows at once (both islands read
//! one signal in the browser), and a dialog confirms what was added.

use crate::cart_totals::TotalRows;
use crate::cart_view::{CartSummary, LineView};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// The newest cart, once something changed it in this page (browser only).
#[cfg(feature = "hydrate")]
static LATEST: std::sync::LazyLock<ArcRwSignal<Option<CartSummary>>> =
    std::sync::LazyLock::new(|| ArcRwSignal::new(None));

/// The cart in the header: a button with the count and a dropdown with the
/// lines and totals.
#[island]
pub fn CartWidget(summary: CartSummary) -> impl IntoView {
    let open = RwSignal::new(false);
    let root = NodeRef::<leptos::html::Div>::new();
    crate::islands::dropdown_behaviour(root, open, false);

    #[cfg(feature = "hydrate")]
    let current = {
        let latest = LATEST.clone();
        Signal::derive(move || latest.get().unwrap_or_else(|| summary.clone()))
    };
    #[cfg(not(feature = "hydrate"))]
    let current = Signal::stored(summary);

    view! {
        <div id="cart-widget" class="cart-dropdown" class:open=open node_ref=root>
            <button
                type="button"
                class="btn-pill cart-btn"
                aria-haspopup="true"
                aria-expanded=move || open.get().to_string()
                aria-label=move || current.with(|s| s.label.clone())
                on:click=move |_| open.update(|o| *o = !*o)
            >
                <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="9" cy="21" r="1"></circle>
                    <circle cx="20" cy="21" r="1"></circle>
                    <path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"></path>
                </svg>
                {move || {
                    let count = current.with(|s| s.count);
                    (count > 0).then(|| view! {
                        <span class="cart-count" aria-hidden="true">{if count > 99 { "99+".to_owned() } else { count.to_string() }}</span>
                    })
                }}
            </button>
            <div class="cart-dropdown-content">
                {move || {
                    let s = current.get();
                    if s.lines.is_empty() {
                        view! { <p class="cart-empty">{s.empty_text}</p> }.into_any()
                    } else {
                        view! {
                            <ul class="cart-items-list">
                                {s.lines.into_iter().map(|line| view! {
                                    <li class="cart-item">
                                        {line.image.clone().map(|src| view! { <img src=src alt=line.description.clone() class="cart-item-img"/> })}
                                        <div class="cart-item-details"><span class="cart-item-desc">{line.description.clone()}</span></div>
                                        <span class="cart-item-price">{line.total}</span>
                                    </li>
                                }).collect_view()}
                            </ul>
                            <div class="cart-totals"><TotalRows rows=s.totals/></div>
                            <a href=s.view_cart_href class="btn-primary full-width cart-dropdown-view-cart">{s.view_cart_text}</a>
                        }
                        .into_any()
                    }
                }}
            </div>
        </div>
    }
}

/// Texts of the add-to-cart button and its dialog.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddTexts {
    pub action: String,
    pub add: String,
    pub decrease: String,
    pub increase: String,
    pub added_title: String,
    pub close: String,
    pub continue_shopping: String,
    pub to_checkout: String,
    pub cart_href: String,
}

/// What came back from adding: the line added and the whole cart.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Added {
    pub line: LineView,
    pub summary: CartSummary,
}

#[server(endpoint = "cart/add")]
pub async fn add_to_cart(variant_id: i64, quantity: u32) -> Result<Added, ServerFnError> {
    let state = expect_context::<crate::state::AppState>();
    let session: tower_sessions::Session = leptos_axum::extract().await?;
    let mut v = crate::visitor::visitor();
    crate::cart_ops::add(&state, &session, &mut v, variant_id, quantity)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let line = crate::cart_view::lines(&v)
        .into_iter()
        .find(|l| l.variant_id == variant_id)
        .ok_or_else(|| ServerFnError::new("not added"))?;
    Ok(Added {
        line,
        summary: crate::cart_view::summary(&v),
    })
}

/// The add-to-cart form of one variant. Without the WebAssembly it posts as
/// a plain form and the page reloads; with it, the item is added in place
/// and a dialog shows it.
#[island]
pub fn AddToCart(variant_id: i64, show_qty: bool, texts: AddTexts) -> impl IntoView {
    let added = RwSignal::new(None::<LineView>);
    let busy = RwSignal::new(false);
    let submit = move |e: leptos::ev::SubmitEvent| {
        e.prevent_default();
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        #[cfg(feature = "hydrate")]
        {
            // The stepper (page behaviour) changes the box directly.
            let typed = e
                .target()
                .and_then(|t| wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlFormElement>(t).ok())
                .and_then(|f| f.query_selector("input[name=quantity]").ok().flatten())
                .and_then(|i| wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlInputElement>(i).ok())
                .and_then(|i| i.value().parse::<u32>().ok())
                .unwrap_or(1);
            leptos::task::spawn_local(async move {
                match add_to_cart(variant_id, typed.max(1)).await {
                    Ok(result) => {
                        LATEST.set(Some(result.summary));
                        added.set(Some(result.line));
                    }
                    Err(_) => {
                        // Fall back to the plain form post.
                        if let Some(form) = leptos::prelude::document()
                            .get_element_by_id(&format!("add-to-cart-{variant_id}"))
                            && let Ok(form) =
                                wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlFormElement>(form)
                        {
                            let _ = form.submit();
                        }
                    }
                }
                busy.set(false);
            });
        }
    };
    let close = move || added.set(None);
    #[cfg(feature = "hydrate")]
    let _ = window_event_listener(leptos::ev::keydown, move |e| {
        if e.key() == "Escape" {
            close();
        }
    });
    let t = texts.clone();
    view! {
        <form action=texts.action.clone() method="post" class="add-to-cart-form" id=format!("add-to-cart-{variant_id}") on:submit=submit>
            <input type="hidden" name="catalog_variant_id" value=variant_id/>
            <div class="qty-cart-row">
                {if show_qty {
                    view! {
                        <div class="qty-stepper">
                            <button type="button" class="qty-btn qty-minus" aria-label=texts.decrease.clone()>"−"</button>
                            <input type="number" name="quantity" value="1" min="1" class="qty-value" readonly/>
                            <button type="button" class="qty-btn qty-plus" aria-label=texts.increase.clone()>"+"</button>
                        </div>
                    }.into_any()
                } else {
                    view! { <input type="hidden" name="quantity" value="1"/> }.into_any()
                }}
                <button type="submit" class="btn-primary btn-cart" title=texts.add.clone() aria-label=texts.add.clone()>
                    <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                        <circle cx="9" cy="21" r="1"></circle>
                        <circle cx="20" cy="21" r="1"></circle>
                        <path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"></path>
                    </svg>
                </button>
            </div>
        </form>
        {move || added.get().map(|line| {
            let t = t.clone();
            view! {
                <leptos::portal::Portal>
                    <div class="cart-modal-backdrop" on:click=move |e| {
                        // Only a click on the backdrop itself closes it.
                        if e.target() == e.current_target() { close(); }
                    }>
                        <div class="cart-modal" role="dialog" aria-modal="true">
                            <button type="button" class="cart-modal-close" aria-label=t.close.clone() on:click=move |_| close()>"✕"</button>
                            <h3 class="cart-modal-title">{t.added_title.clone()}</h3>
                            <hr class="cart-modal-divider"/>
                            <div class="cart-modal-item">
                                {line.image.clone().map(|src| view! { <img src=src alt=line.description.clone() class="cart-modal-item-img"/> })}
                                <div class="cart-modal-item-details">
                                    <span class="cart-modal-item-desc">{line.description.clone()}</span>
                                    <span class="cart-modal-item-price">{line.total.clone()}</span>
                                </div>
                            </div>
                            <div class="cart-modal-actions">
                                <button type="button" class="btn-outline-dark" on:click=move |_| close()>{t.continue_shopping.clone()}</button>
                                <a href=t.cart_href.clone() class="btn-primary cart-modal-checkout">
                                    {t.to_checkout.clone()}
                                    <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>
                                </a>
                            </div>
                        </div>
                    </div>
                </leptos::portal::Portal>
            }
        })}
    }
}
