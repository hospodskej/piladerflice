//! /muj-ucet: the account overview and the last orders.

use leptos::prelude::*;

use super::{AccountView, Avatar};
use crate::ui::{CsrfField, path, t, t_with};

pub struct OrderCard {
    pub id: i64,
    /// "14.10.2026"
    pub date: String,
    pub items: usize,
    pub total: String,
}

#[component]
pub fn AccountPage(account: AccountView, orders: Vec<OrderCard>) -> impl IntoView {
    view! {
        <main class="main">
            <section class="account-dashboard">
                <header class="account-profile-card">
                    <Avatar avatar_path=account.avatar_path initial=account.initial/>
                    <div class="account-profile-info">
                        <h1 class="account-profile-title">
                            {t("auth.account_title")}
                            {account
                                .is_admin
                                .then(|| view! { " "<span class="account-admin-badge">{t("auth.admin_badge")}</span> })}
                        </h1>
                        <p class="account-profile-email">{account.email}</p>
                        <p class="account-profile-since">{t_with("auth.customer_since", &[("date", &account.since)])}</p>
                    </div>
                    <div class="account-profile-actions">
                        {account
                            .is_admin
                            .then(|| view! { <a class="btn-primary account-action-btn" href="/admin">{t("auth.go_to_admin")}</a>" " })}
                        <a class="btn-outline-dark account-action-btn" href=path("/muj-ucet/upravit")>{t("auth.edit_profile_link")}</a>
                        " "
                        <form class="button_to" method="post" action=path("/odhlaseni")>
                            <button class="btn-outline-dark account-action-btn" type="submit">{t("auth.logout")}</button>
                            <CsrfField/>
                        </form>
                    </div>
                </header>
                <Orders orders=orders/>
            </section>
        </main>
    }
}

#[component]
fn Orders(orders: Vec<OrderCard>) -> impl IntoView {
    let count = orders.len();
    let list = if orders.is_empty() {
        view! {
            <div class="account-orders-empty">
                <p>{t("auth.no_orders")}</p>
                <a class="btn-primary" href=path("/eshop")>{t("nav.eshop")}</a>
            </div>
        }
        .into_any()
    } else {
        view! {
            <div class="order-card-list">
                {orders
                    .into_iter()
                    .map(|order| {
                        let items = if order.items == 1 {
                            t("auth.order_item_count_one")
                        } else {
                            t_with("auth.order_item_count_other", &[("count", &order.items.to_string())])
                        };
                        view! {
                            <div class="order-card">
                                <div class="order-card-main">
                                    <span class="order-card-number">{format!("{} {}", t("auth.order_number"), order.id)}</span>
                                    " "
                                    <span class="order-card-meta">{format!("{} · {items}", order.date)}</span>
                                </div>
                                " "
                                <span class="order-card-total">{order.total}</span>
                                " "
                                <form
                                    class="button_to"
                                    method="post"
                                    action=path(&format!("/muj-ucet/objednavky/{}/znovu-objednat", order.id))
                                >
                                    <button class="btn-outline-dark order-card-reorder-btn" type="submit">{t("auth.reorder_button")}</button>
                                    <CsrfField/>
                                </form>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        }
        .into_any()
    };
    view! {
        <section class="account-orders-section">
            <div class="account-orders-heading">
                <h2>{t("auth.order_history_title")}</h2>
                {(count > 0).then(|| view! { " "<span class="account-orders-count">{count}</span> })}
            </div>
            {list}
        </section>
    }
}
