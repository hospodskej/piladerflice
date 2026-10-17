//! The menu that slides in on phones.

use leptos::prelude::*;

use super::header::{NavLinks, ThemeToggle};
use crate::ui::icons::{MailIcon, PhoneIcon};
use crate::ui::{asset, path, t};

#[component]
pub fn MobileSidebar() -> impl IntoView {
    view! {
        <div class="sidebar-overlay" id="sidebarOverlay"></div>
        <aside class="mobile-sidebar" id="mobileSidebar">
            <div class="sidebar-header">
                <a aria-label=t("nav.domu") href=path("/")>
                    <img alt=t("site.name") class="sidebar-logo" src=asset("logo-wm.webp")/>
                </a>
                <button class="close-sidebar" id="closeSidebar" aria-label=t("nav.close_menu")>"✕"</button>
            </div>
            <nav class="sidebar-nav">
                <NavLinks/>
            </nav>
            <div class="sidebar-bottom">
                <div class="sidebar-theme-toggle">
                    <span>{t("nav.dark_mode_label")}</span>
                    <ThemeToggle id="themeToggleMobile"/>
                </div>
                <div class="sidebar-contact">
                    <div class="contact-item">
                        <PhoneIcon size=18/>
                        " "
                        <a href="tel:+420602446339" class="contact-link">"+420 602 446 339"</a>
                    </div>
                    <div class="contact-item">
                        <MailIcon size=14/>
                        " "
                        <a href="mailto:stepan.merta@seznam.cz" class="contact-link">"stepan.merta@seznam.cz"</a>
                    </div>
                </div>
            </div>
        </aside>
    }
}
