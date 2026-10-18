//! The cart's islands: the header widget, the add-to-cart buttons, the
//! "added to cart" window and the cart page's table. A change goes to the
//! server through a server function, which answers with the new cart; every
//! island showing the cart then updates ([`shared::cart`]).
//!
//! Each island is also a plain form (the form token, an action), so the
//! cart works before the WebAssembly has loaded, or without it.

use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

use super::shared;
use crate::icons::CartIcon;

/// One cart line, described in the page's language and currency.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LineView {
    pub id: String,
    pub description: String,
    pub image: Option<String>,
    pub quantity: i64,
    pub unit_price: String,
    pub total_excl_vat: String,
    pub total: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CartView {
    pub empty: bool,
    pub lines: Vec<LineView>,
    pub subtotal: String,
    pub vat: String,
    pub excl_vat: String,
}

/// The answer to "add to cart": the line added and the whole cart.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Added {
    pub line: LineView,
    pub cart: CartView,
}

/// Puts `quantity` of a variant into the cart. `locale` is the page's
/// language ("cs", "de"): the answer is described in it.
#[server(prefix = "/api", endpoint = "cart/add")]
pub async fn add_to_cart(locale: String, variant_id: i64, quantity: i64) -> Result<Added, ServerFnError> {
    use super::server::failed;
    use crate::shop::adding::put_in_cart;
    use crate::views::cart_view;

    let (state, req, locale) = super::server::request(&locale).await?;
    let line = put_in_cart(&state, &req, variant_id, quantity)
        .await
        .map_err(failed)?
        .ok_or_else(|| ServerFnError::new("this variant can't be bought"))?;
    Ok(Added {
        line: cart_view::line(&state, locale, &line).await.map_err(failed)?,
        cart: cart_now(&state, &req, locale).await?,
    })
}

/// Sets a line's quantity (less than 1 is 1).
#[server(prefix = "/api", endpoint = "cart/update")]
pub async fn update_cart_line(locale: String, line_id: String, quantity: i64) -> Result<CartView, ServerFnError> {
    let (state, req, locale) = super::server::request(&locale).await?;
    req.session.update(|data| data.cart.update_quantity(&line_id, quantity));
    cart_now(&state, &req, locale).await
}

#[server(prefix = "/api", endpoint = "cart/remove")]
pub async fn remove_cart_line(locale: String, line_id: String) -> Result<CartView, ServerFnError> {
    let (state, req, locale) = super::server::request(&locale).await?;
    req.session.update(|data| data.cart.remove(&line_id));
    cart_now(&state, &req, locale).await
}

/// The session's cart, for the answer.
#[cfg(feature = "ssr")]
async fn cart_now(
    state: &crate::AppState,
    req: &crate::request::Req,
    locale: crate::locale::Locale,
) -> Result<CartView, ServerFnError> {
    let cart = req.session.read(|data| data.cart.clone());
    crate::views::cart_view::build(state, locale, &cart)
        .await
        .map_err(super::server::failed)
}

/// The cart as it is now: as rendered, or as the last change left it.
fn current(rendered: CartView) -> impl Fn() -> CartView + Copy + Send + Sync + 'static {
    let rendered = StoredValue::new(rendered);
    let changed = shared::cart();
    move || changed.get().unwrap_or_else(|| rendered.get_value())
}

/// The words of the header widget.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WidgetTexts {
    pub aria_label: String,
    pub empty: String,
    pub total_due: String,
    pub without_vat: String,
    pub view_cart: String,
    pub cart_href: String,
}

/// The cart button in the header and the list that opens under it.
#[island]
pub fn CartWidget(cart: CartView, texts: WidgetTexts) -> impl IntoView {
    let cart = current(cart);
    let texts = StoredValue::new(texts);
    let contents = move || {
        let cart = cart();
        let texts = texts.get_value();
        if cart.empty {
            return view! { <p class="cart-empty">{texts.empty}</p> }.into_any();
        }
        view! {
            <ul class="cart-items-list">
                {cart
                    .lines
                    .into_iter()
                    .map(|item| {
                        view! {
                            <li class="cart-item">
                                {item.image.map(|src| view! { <img alt=item.description.clone() class="cart-item-img" src=src/> })}
                                <div class="cart-item-details">
                                    <span class="cart-item-desc">{item.description}</span>
                                </div>
                                <span class="cart-item-price">{item.total}</span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
            <div class="cart-totals">
                <div class="cart-total-row">
                    <span>{texts.total_due}</span>
                    <strong>{cart.subtotal}</strong>
                </div>
                <div class="cart-total-row cart-total-excl-vat">
                    <span>{texts.without_vat}</span>
                    <span>{cart.excl_vat}</span>
                </div>
            </div>
            <a class="btn-primary full-width cart-dropdown-view-cart" href=texts.cart_href>{texts.view_cart}</a>
        }
        .into_any()
    };
    view! {
        <div id="cart-widget" class="cart-dropdown">
            <button type="button" class="btn-pill cart-btn" aria-label=texts.get_value().aria_label>
                <CartIcon/>
            </button>
            <div class="cart-dropdown-content">{contents}</div>
        </div>
    }
}

/// The words and the form of an add-to-cart button.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BuyTexts {
    pub locale: String,
    /// Where the plain form posts.
    pub action: String,
    pub add: String,
    pub decrease: String,
    pub increase: String,
}

/// Puts a variant into the cart; `show_qty`: with the quantity stepper.
/// `children`: the form's hidden fields (the form token, the variant).
#[island]
pub fn AddToCart(variant_id: i64, show_qty: bool, texts: BuyTexts, children: Children) -> impl IntoView {
    let quantity = RwSignal::new(1_i64);
    let locale = texts.locale.clone();
    let submit = move |event: SubmitEvent| {
        event.prevent_default();
        let form = event_target::<web_sys::HtmlFormElement>(&event);
        let locale = locale.clone();
        spawn_local(async move {
            match add_to_cart(locale, variant_id, quantity.get_untracked()).await {
                Ok(added) => {
                    shared::cart().set(Some(added.cart));
                    shared::added().set(Some(added.line));
                }
                // Let the server answer the plain form (e.g. sold out since).
                Err(_) => {
                    let _ = form.submit();
                }
            }
        });
    };
    let stepper = if show_qty {
        view! {
            <div class="qty-stepper">
                <button
                    type="button"
                    class="qty-btn qty-minus"
                    aria-label=texts.decrease
                    on:click=move |_| quantity.update(|n| *n = (*n - 1).max(1))
                >
                    "−"
                </button>
                <input
                    type="number"
                    name="quantity"
                    id="quantity"
                    value=move || quantity.get()
                    min="1"
                    class="qty-value"
                    readonly="readonly"
                />
                <button type="button" class="qty-btn qty-plus" aria-label=texts.increase on:click=move |_| quantity.update(|n| *n += 1)>
                    "+"
                </button>
            </div>
        }
        .into_any()
    } else {
        view! { <input type="hidden" name="quantity" id="quantity" value="1"/> }.into_any()
    };
    view! {
        <form class="add-to-cart-form" action=texts.action accept-charset="UTF-8" method="post" on:submit=submit>
            {children()}
            <div class="qty-cart-row">
                {stepper}
                <button type="submit" class="btn-primary btn-cart" title=texts.add.clone() aria-label=texts.add>
                    <CartIcon aria_hidden=true/>
                </button>
            </div>
        </form>
    }
}

/// The words of the "added to cart" window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModalTexts {
    pub close: String,
    pub title: String,
    pub continue_shopping: String,
    pub to_checkout: String,
    pub cart_href: String,
}

/// The window shown after "add to cart"; closed with its buttons, a click
/// beside it or Escape.
#[island]
pub fn CartAddedModal(texts: ModalTexts) -> impl IntoView {
    let added = shared::added();
    let close = move || added.set(None);
    let handle = window_event_listener(leptos::ev::keydown, move |event| {
        if event.key() == "Escape" {
            close();
        }
    });
    on_cleanup(move || handle.remove());
    let texts = StoredValue::new(texts);
    move || {
        let item = added.get()?;
        let texts = texts.get_value();
        Some(view! {
            <div
                class="cart-modal-backdrop"
                on:click=move |event| {
                    if event.target() == event.current_target() {
                        close();
                    }
                }
            >
                <div class="cart-modal" role="dialog" aria-modal="true">
                    <button type="button" class="cart-modal-close" aria-label=texts.close on:click=move |_| close()>"✕"</button>
                    <h3 class="cart-modal-title">{texts.title}</h3>
                    <hr class="cart-modal-divider"/>
                    <div class="cart-modal-item">
                        {item.image.map(|src| view! { <img alt=item.description.clone() class="cart-modal-item-img" src=src/> })}
                        <div class="cart-modal-item-details">
                            <span class="cart-modal-item-desc">{item.description.clone()}</span>
                            " "
                            <span class="cart-modal-item-price">{item.total}</span>
                        </div>
                    </div>
                    <div class="cart-modal-actions">
                        <button type="button" class="btn-outline-dark" on:click=move |_| close()>{texts.continue_shopping}</button>
                        " "
                        <a class="btn-primary cart-modal-checkout" href=texts.cart_href>
                            {texts.to_checkout}
                            " "
                            <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <polyline points="9 18 15 12 9 6"/>
                            </svg>
                        </a>
                    </div>
                </div>
            </div>
        })
    }
}

/// The words and links of the cart page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PageTexts {
    pub locale: String,
    pub csrf_token: String,
    /// "/cart_items" in the page's language: the plain forms post to
    /// `{lines_action}/{id}` and `{lines_action}/{id}/delete`.
    pub lines_action: String,
    pub empty: String,
    pub eshop: String,
    pub eshop_href: String,
    pub contents: String,
    pub quantity: String,
    pub availability: String,
    pub unit_price: String,
    pub price_excl_vat: String,
    pub price_incl_vat: String,
    pub in_stock: String,
    pub remove: String,
    pub discount_code: String,
    pub discount_verify: String,
    pub total_due: String,
    pub total_vat: String,
    pub without_vat: String,
    pub back_to_shopping: String,
    pub continue_: String,
    pub shipping_href: String,
}

/// The cart table with the totals; quantities change and lines go away in
/// place.
#[island]
pub fn CartContents(cart: CartView, texts: PageTexts) -> impl IntoView {
    let cart = current(cart);
    let texts = StoredValue::new(texts);
    let contents = move || {
        let cart = cart();
        let texts = texts.get_value();
        let line_texts = texts.clone();
        if cart.empty {
            return view! {
                <p class="cart-empty">{texts.empty}</p>
                " "
                <a class="btn-primary" href=texts.eshop_href>{texts.eshop}</a>
            }
            .into_any();
        }
        view! {
            <div class="checkout-cart-table">
                <div class="checkout-cart-row checkout-cart-header">
                    <span class="col-item">{texts.contents}</span>
                    <span class="col-qty">{texts.quantity}</span>
                    <span class="col-avail">{texts.availability}</span>
                    <span class="col-unit-price">{texts.unit_price}</span>
                    <span class="col-price-excl">{texts.price_excl_vat}</span>
                    <span class="col-price-incl">{texts.price_incl_vat}</span>
                    <span class="col-remove"></span>
                </div>
                {cart.lines.into_iter().map(|line| view! { <Line line=line texts=line_texts.clone()/> }).collect_view()}
            </div>
            <div class="checkout-cart-footer">
                <div class="discount-code-box">
                    <label for="discount-code">{texts.discount_code}</label>
                    <div class="discount-code-row">
                        <input type="text" id="discount-code" name="discount-code" disabled=true/>
                        <button type="button" class="btn-primary" disabled=true>{texts.discount_verify}</button>
                    </div>
                </div>
                <div class="cart-totals checkout-cart-totals">
                    <div class="cart-total-row">
                        <span>{texts.total_due}</span>
                        <strong>{cart.subtotal}</strong>
                    </div>
                    <div class="cart-total-row cart-total-excl-vat">
                        <span>{texts.total_vat}</span>
                        <span>{cart.vat}</span>
                    </div>
                    <div class="cart-total-row cart-total-excl-vat">
                        <span>{texts.without_vat}</span>
                        <span>{cart.excl_vat}</span>
                    </div>
                </div>
            </div>
            <div class="checkout-nav">
                <a class="btn-outline-dark" href=texts.eshop_href>{texts.back_to_shopping}</a>
                " "
                <a class="btn-primary" href=texts.shipping_href>{texts.continue_}</a>
            </div>
        }
        .into_any()
    };
    view! { <div id="cart-page-contents">{contents}</div> }
}

/// Puts the server's answer into the page, or reloads it when there is none.
fn show_cart(answer: Result<CartView, ServerFnError>) {
    match answer {
        Ok(cart) => shared::cart().set(Some(cart)),
        Err(_) => {
            let _ = window().location().reload();
        }
    }
}

#[component]
fn Line(line: LineView, texts: PageTexts) -> impl IntoView {
    let change = {
        let (locale, id) = (texts.locale.clone(), line.id.clone());
        move |event| {
            let quantity = event_target_value(&event).trim().parse::<i64>().unwrap_or(0);
            let (locale, id) = (locale.clone(), id.clone());
            spawn_local(async move { show_cart(update_cart_line(locale, id, quantity).await) });
        }
    };
    let remove = {
        let (locale, id) = (texts.locale.clone(), line.id.clone());
        move |event: SubmitEvent| {
            event.prevent_default();
            let (locale, id) = (locale.clone(), id.clone());
            spawn_local(async move { show_cart(remove_cart_line(locale, id).await) });
        }
    };
    let token = move || view! { <input type="hidden" name="csrf_token" value=texts.csrf_token.clone()/> };
    view! {
        <div class="checkout-cart-row">
            <div class="col-item">
                {line.image.map(|src| view! { <img alt=line.description.clone() class="cart-item-img" src=src/> })}
                <span class="cart-item-desc">{line.description.clone()}</span>
            </div>
            <div class="col-qty">
                <form action=format!("{}/{}", texts.lines_action, line.id) accept-charset="UTF-8" method="post">
                    {token()}
                    <input
                        type="number"
                        name="quantity"
                        value=line.quantity
                        min="1"
                        step="1"
                        class="qty-input"
                        aria-label=texts.quantity
                        on:change=change
                    />
                </form>
            </div>
            <div class="col-avail">
                <span class="text-green">{texts.in_stock}</span>
            </div>
            <div class="col-unit-price">{line.unit_price}</div>
            <div class="col-price-excl">{line.total_excl_vat}</div>
            <div class="col-price-incl">
                <strong>{line.total}</strong>
            </div>
            <div class="col-remove">
                <form class="button_to" method="post" action=format!("{}/{}/delete", texts.lines_action, line.id) on:submit=remove>
                    <button class="cart-remove-btn" aria-label=texts.remove type="submit">"×"</button>
                    {token()}
                </form>
            </div>
        </div>
    }
}

/// The texts, from the page's translations (on the server only).
#[cfg(feature = "ssr")]
mod texts {
    use super::*;
    use crate::ui::{csrf_token, locale, path, t};

    impl WidgetTexts {
        pub fn translated() -> Self {
            Self {
                aria_label: t("cart.aria_label"),
                empty: t("cart.empty"),
                total_due: t("cart.total_due"),
                without_vat: t("cart.without_vat"),
                view_cart: t("cart.view_cart"),
                cart_href: path("/kosik"),
            }
        }
    }

    impl BuyTexts {
        pub fn translated() -> Self {
            Self {
                locale: locale().code().to_string(),
                action: path("/cart_items"),
                add: t("common.add_to_cart"),
                decrease: t("common.qty_decrease"),
                increase: t("common.qty_increase"),
            }
        }
    }

    impl ModalTexts {
        pub fn translated() -> Self {
            Self {
                close: t("common.close"),
                title: t("cart.added_title"),
                continue_shopping: t("cart.continue_shopping"),
                to_checkout: t("cart.to_checkout"),
                cart_href: path("/kosik"),
            }
        }
    }

    impl PageTexts {
        pub fn translated() -> Self {
            Self {
                locale: locale().code().to_string(),
                csrf_token: csrf_token(),
                lines_action: path("/cart_items"),
                empty: t("cart.empty"),
                eshop: t("nav.eshop"),
                eshop_href: path("/eshop"),
                contents: t("cart.table_contents"),
                quantity: t("cart.table_quantity"),
                availability: t("cart.table_availability"),
                unit_price: t("cart.table_unit_price"),
                price_excl_vat: t("cart.table_price_excl_vat"),
                price_incl_vat: t("cart.table_price_incl_vat"),
                in_stock: t("common.in_stock"),
                remove: t("cart.remove_aria"),
                discount_code: t("cart.discount_code_label"),
                discount_verify: t("cart.discount_code_verify"),
                total_due: t("cart.total_due"),
                total_vat: t("cart.total_vat"),
                without_vat: t("cart.without_vat"),
                back_to_shopping: t("cart.back_to_shopping"),
                continue_: t("checkout.continue"),
                shipping_href: path("/kosik/doprava"),
            }
        }
    }
}
