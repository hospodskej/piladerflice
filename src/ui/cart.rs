//! The cart page (checkout step 1). The table itself is an island
//! (`islands/cart.rs`): quantities change in place.

use leptos::prelude::*;

use super::checkout::Stepper;
use super::render::current_cart;
use crate::islands::cart::{CartContents, PageTexts};

#[component]
pub fn CartPage() -> impl IntoView {
    view! {
        <main class="main">
            <section class="checkout-section">
                <Stepper current=1/>
                <CartContents cart=current_cart() texts=PageTexts::translated()/>
            </section>
        </main>
    }
}
