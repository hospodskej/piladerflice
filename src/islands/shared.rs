//! State that several islands on a page share: the cart (the header widget
//! and the cart page), the "added to cart" window, the mobile menu, the
//! cookie banner, the product filter.
//!
//! In the browser there is one of each for the whole page. On the server
//! every island renders from its own props, so each call gets a fresh
//! signal: nothing is ever shared between requests.

use leptos::prelude::*;

use super::cart::{CartView, LineView};

macro_rules! shared {
    ($($(#[$doc:meta])* $name:ident: $type:ty = $init:expr;)*) => {
        $(
            $(#[$doc])*
            pub fn $name() -> RwSignal<$type> {
                #[cfg(feature = "hydrate")]
                {
                    thread_local! {
                        static SIGNAL: RwSignal<$type> = RwSignal::new($init);
                    }
                    SIGNAL.with(|signal| *signal)
                }
                #[cfg(not(feature = "hydrate"))]
                {
                    RwSignal::new($init)
                }
            }
        )*
    };
}

shared! {
    /// The cart after a change made on this page; None until then (the
    /// islands show the cart they were rendered with).
    cart: Option<CartView> = None;
    /// The line just added, shown in the "added to cart" window.
    added: Option<LineView> = None;
    /// The mobile menu is open.
    menu_open: bool = false;
    /// The cookie banner is shown (after "Nastavení cookies" in the footer).
    cookie_banner: bool = false;
    /// The product filter of the page: "all", or what to show only
    /// ("kontejner", a lumber dimension...).
    filter: String = "all".to_string();
    /// The category of the quote request form: "palivove" or "stavebni".
    quote_category: String = "palivove".to_string();
    /// The quote form's list whose menu is open, if any.
    open_menu: Option<u64> = None;
}
