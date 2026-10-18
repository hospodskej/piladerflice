//! /admin: counts, and the latest orders.

use leptos::prelude::*;

use super::orders::{OrderRow, OrdersTable};

pub struct Dashboard {
    pub products: i64,
    pub orders: i64,
    pub orders_this_month: i64,
    pub orders_last_month: i64,
    pub recent: Vec<OrderRow>,
}

#[component]
pub fn DashboardPage(dashboard: Dashboard) -> impl IntoView {
    let card = |number: i64, label: &'static str, href: &'static str| {
        view! {
            <a class="admin-dashboard-card" href=href>
                <span class="admin-dashboard-card-number">{number}</span>
                <span class="admin-dashboard-card-label">{label}</span>
            </a>
        }
    };
    let recent = if dashboard.recent.is_empty() {
        view! { <p>"Zatím žádné objednávky."</p> }.into_any()
    } else {
        view! {
            <OrdersTable orders=dashboard.recent full=false/>
            <a href="/admin/orders">"Zobrazit všechny objednávky →"</a>
        }
        .into_any()
    };
    view! {
        <h1 class="admin-page-title">"Přehled"</h1>
        <div class="admin-dashboard-cards">
            {card(dashboard.products, "produktů v katalogu", "/admin/catalog_products")}
            {card(dashboard.orders, "objednávek celkem", "/admin/orders")}
            {card(dashboard.orders_this_month, "objednávky tento měsíc", "/admin/orders")}
            {card(dashboard.orders_last_month, "objednávky minulý měsíc", "/admin/orders")}
        </div>
        <h2 class="admin-section-title">"Poslední objednávky"</h2>
        {recent}
    }
}
