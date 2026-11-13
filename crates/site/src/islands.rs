//! The interactive parts of the pages. Each island is rendered on the server
//! with the rest of the page and wakes up in the browser (WASM); everything
//! else on the page is plain HTML.

use leptos::prelude::*;

/// A header dropdown (language/currency, cart). Its children are the button
/// (first) and what opens. A mouse still opens it on hover (CSS); a click, a
/// tap or Enter/Space on the button toggles it, and a click elsewhere, Tab
/// out of it or Escape closes it.
#[island]
pub fn Dropdown(
    /// The wrapper's class: `lang-dropdown` or `cart-dropdown`.
    class: String,
    #[prop(optional)] id: Option<String>,
    children: Children,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let root = NodeRef::<leptos::html::Div>::new();

    #[cfg(feature = "hydrate")]
    {
        use leptos::ev;
        use wasm_bindgen::JsCast;
        let button = move || {
            root.get_untracked()
                .and_then(|r| r.query_selector(":scope > button").ok().flatten())
                .and_then(|b| b.dyn_into::<web_sys::HtmlElement>().ok())
        };
        Effect::new(move |_| {
            let is_open = open.get();
            if let Some(b) = button() {
                let _ = b.set_attribute("aria-expanded", if is_open { "true" } else { "false" });
            }
        });
        let inside = move |target: Option<web_sys::EventTarget>| {
            let node = target.and_then(|t| t.dyn_into::<web_sys::Node>().ok());
            root.get_untracked().is_some_and(|r| r.contains(node.as_ref()))
        };
        let on_button = move |target: Option<web_sys::EventTarget>| {
            let node = target.and_then(|t| t.dyn_into::<web_sys::Node>().ok());
            button().is_some_and(|b| b.contains(node.as_ref()))
        };
        let _ = window_event_listener(ev::click, move |e| {
            if on_button(e.target()) {
                open.update(|o| *o = !*o);
            } else if open.get_untracked() && !inside(e.target()) {
                open.set(false);
            }
        });
        let _ = window_event_listener(ev::focusin, move |e| {
            if open.get_untracked() && !inside(e.target()) {
                open.set(false);
            }
        });
        let _ = window_event_listener(ev::keydown, move |e| {
            if e.key() == "Escape" && open.get_untracked() {
                open.set(false);
                if let Some(b) = button() {
                    let _ = b.focus();
                }
            }
        });
    }

    view! {
        <div class=class id=id class:open=open node_ref=root>
            {children()}
        </div>
    }
}

/// The dark/light switch. The choice is kept on this device (localStorage);
/// for a signed-in customer it's also saved to the account, so another
/// device signed in to the same account follows. The page applies it before
/// the first paint (a small inline script in the shell).
#[island]
pub fn ThemeToggle(
    #[prop(optional)] id: Option<String>,
    label: String,
    /// Save the choice to the account too.
    signed_in: bool,
) -> impl IntoView {
    let toggle = move |_| {
        #[cfg(feature = "hydrate")]
        {
            let theme = if browser::html_has_class("dark-mode") { "light" } else { "dark" };
            browser::apply_theme(theme);
            if signed_in {
                leptos::task::spawn_local(async move {
                    let _ = crate::server_fns::save_theme(theme.to_owned()).await;
                });
            }
        }
        let _ = signed_in;
    };
    view! {
        <button type="button" class="theme-toggle" id=id aria-label=label on:click=toggle>
            <span class="theme-toggle-track">
                <svg class="theme-icon theme-icon-sun" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4"></circle><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"></path></svg>
                <svg class="theme-icon theme-icon-moon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path></svg>
                <span class="theme-toggle-thumb"></span>
            </span>
        </button>
    }
}

/// Behaviour that ties together elements spread over the page, so it can't
/// live in one island: the mobile menu (button in the header, sidebar at the
/// end of the page), the sticky logo bar, the Ceník/Kontakt highlight on the
/// contact page, the cookie banner (and the footer button that reopens it),
/// accordions and quantity steppers. Rendered once, at the end of the page.
#[island]
pub fn PageBehaviour() -> impl IntoView {
    #[cfg(feature = "hydrate")]
    browser::install_page_behaviour();
}

#[cfg(feature = "hydrate")]
pub(crate) mod browser {
    use leptos::ev;
    use leptos::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::{Element, HtmlElement, HtmlInputElement};

    const CONSENT_COOKIE: &str = "cookie_consent";
    const CONSENT_MAX_AGE: u32 = 180 * 24 * 60 * 60;

    fn document() -> web_sys::Document {
        leptos::prelude::document()
    }

    fn by_id(id: &str) -> Option<Element> {
        document().get_element_by_id(id)
    }

    fn target_closest(event: &web_sys::Event, selector: &str) -> Option<Element> {
        event.target()?.dyn_into::<Element>().ok()?.closest(selector).ok().flatten()
    }

    pub fn html_has_class(class: &str) -> bool {
        document().document_element().is_some_and(|e| e.class_list().contains(class))
    }

    pub fn apply_theme(theme: &str) {
        if let Some(html) = document().document_element() {
            let _ = html.class_list().toggle_with_force("dark-mode", theme == "dark");
        }
        if let Ok(Some(storage)) = window().local_storage() {
            let _ = storage.set_item("theme", theme);
        }
    }

    fn set_hidden(id: &str, hidden: bool) {
        if let Some(el) = by_id(id).and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
            el.set_hidden(hidden);
        }
    }

    fn consent_cookie() -> Option<String> {
        let cookies = document().dyn_into::<web_sys::HtmlDocument>().ok()?.cookie().ok()?;
        cookies
            .split("; ")
            .find_map(|c| c.strip_prefix(CONSENT_COOKIE)?.strip_prefix('=').map(str::to_owned))
    }

    fn set_consent_cookie(choice: &str) {
        if let Ok(doc) = document().dyn_into::<web_sys::HtmlDocument>() {
            let _ = doc.set_cookie(&format!("{CONSENT_COOKIE}={choice}; path=/; max-age={CONSENT_MAX_AGE}; SameSite=Lax"));
        }
    }

    /// `gtag("consent", "update", {analytics_storage: ...})`, if Analytics loaded.
    fn update_analytics_consent(granted: bool) {
        let Ok(gtag) = js_sys::Reflect::get(&window(), &"gtag".into()) else { return };
        let Ok(gtag) = gtag.dyn_into::<js_sys::Function>() else { return };
        let settings = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&settings, &"analytics_storage".into(), &(if granted { "granted" } else { "denied" }).into());
        let _ = gtag.call3(&wasm_bindgen::JsValue::NULL, &"consent".into(), &"update".into(), &settings);
    }

    fn set_sidebar_open(open: bool) {
        for id in ["mobileSidebar", "sidebarOverlay"] {
            if let Some(el) = by_id(id) {
                let _ = el.class_list().toggle_with_force("active", open);
            }
        }
        if let Some(body) = document().body() {
            let _ = body.style().set_property("overflow", if open { "hidden" } else { "" });
        }
    }

    fn update_sticky_logo() {
        let (Some(bar), Some(header)) = (by_id("stickyLogoBar"), document().query_selector(".site-header").ok().flatten()) else {
            return;
        };
        let header_height = header.dyn_into::<HtmlElement>().map(|h| h.offset_height()).unwrap_or(0);
        let scrolled = window().scroll_y().unwrap_or(0.0);
        let _ = bar.class_list().toggle_with_force("is-visible", scrolled > f64::from(header_height));
    }

    /// On the contact page the Ceník link is active when the price list is
    /// in view (#cenik), the Kontakt link otherwise.
    fn update_nav_highlight() {
        let path = window().location().pathname().unwrap_or_default();
        if path.trim_end_matches('/') != "/kontakt" && path.trim_end_matches('/') != "/at/kontakt" {
            return;
        }
        let on_pricelist = window().location().hash().unwrap_or_default() == "#cenik";
        if let Ok(links) = document().query_selector_all("#nav-cenik, #nav-kontakt") {
            for i in 0..links.length() {
                if let Some(link) = links.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                    let is_cenik = link.id() == "nav-cenik";
                    let _ = link.class_list().toggle_with_force("active", is_cenik == on_pricelist);
                }
            }
        }
    }

    pub fn install_page_behaviour() {
        if consent_cookie().is_none() {
            set_hidden("cookie-consent-banner", false);
        }
        sync_theme_with_account();
        update_sticky_logo();
        update_nav_highlight();

        let _ = window_event_listener(ev::scroll, |_| update_sticky_logo());
        let _ = window_event_listener(ev::resize, |_| update_sticky_logo());
        let _ = window_event_listener(ev::hashchange, |_| update_nav_highlight());
        let _ = window_event_listener(ev::click, |event| {
            let event: &web_sys::Event = &event;
            if target_closest(event, "#menuToggle").is_some() {
                set_sidebar_open(true);
            } else if target_closest(event, "#closeSidebar, #sidebarOverlay").is_some() {
                set_sidebar_open(false);
            } else if let Some(choice) = target_closest(event, "[data-cookie-consent]") {
                let choice = choice.get_attribute("data-cookie-consent").unwrap_or_default();
                set_consent_cookie(&choice);
                set_hidden("cookie-consent-banner", true);
                update_analytics_consent(choice == "all");
            } else if target_closest(event, "[data-cookie-settings]").is_some() {
                event.prevent_default();
                set_hidden("cookie-consent-banner", false);
            } else if let Some(header) = target_closest(event, ".accordion-header") {
                if let Some(item) = header.parent_element() {
                    let _ = item.class_list().toggle("active");
                }
            } else if let Some(button) = target_closest(event, ".qty-plus, .qty-minus") {
                step_quantity(&button);
            }
        });
    }

    fn step_quantity(button: &Element) {
        let Some(input) = button
            .closest(".qty-cart-row")
            .ok()
            .flatten()
            .and_then(|row| row.query_selector(".qty-value").ok().flatten())
            .and_then(|i| i.dyn_into::<HtmlInputElement>().ok())
        else {
            return;
        };
        let min: i64 = input.min().parse().unwrap_or(1);
        let current: i64 = input.value().parse().unwrap_or(min);
        let next = if button.class_list().contains("qty-plus") { current + 1 } else { (current - 1).max(min) };
        input.set_value(&next.to_string());
    }

    /// A signed-in account without a saved choice takes this device's.
    fn sync_theme_with_account() {
        let Some(meta) = document().query_selector(r#"meta[name="theme-preference"]"#).ok().flatten() else {
            return;
        };
        let saved = meta.get_attribute("content").unwrap_or_default();
        if !saved.is_empty() {
            apply_theme(&saved);
            return;
        }
        let local = window().local_storage().ok().flatten().and_then(|s| s.get_item("theme").ok().flatten());
        if let Some(theme) = local {
            leptos::task::spawn_local(async move {
                let _ = crate::server_fns::save_theme(theme).await;
            });
        }
    }
}
