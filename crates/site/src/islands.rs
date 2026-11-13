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
                .and_then(|r| r.query_selector(":scope > button, :scope > leptos-children > button").ok().flatten())
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

    /// The elements under `root` matching `selector`.
    pub fn all(root: &Element, selector: &str) -> Vec<Element> {
        let Ok(list) = root.query_selector_all(selector) else { return Vec::new() };
        (0..list.length()).filter_map(|i| list.item(i)?.dyn_into::<Element>().ok()).collect()
    }

    /// The element matching `selector` that the event happened in, if it's inside `root`.
    pub fn closest_within(event: &web_sys::Event, root: &Element, selector: &str) -> Option<Element> {
        let found = target_closest(event, selector)?;
        root.contains(Some(&found)).then_some(found)
    }

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

        fit_lines();
        let _ = window_event_listener(ev::scroll, |_| update_sticky_logo());
        let _ = window_event_listener(ev::resize, |_| {
            update_sticky_logo();
            fit_lines();
        });
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

    /// Where the cards are: inside the island's <leptos-children> wrapper
    /// (`display: contents`, so the track still lays them out).
    fn cards_parent(track: &Element) -> Element {
        track.query_selector(":scope > leptos-children").ok().flatten().unwrap_or_else(|| track.clone())
    }

    fn real_cards(track: &Element) -> Vec<Element> {
        let children = cards_parent(track).children();
        (0..children.length())
            .filter_map(|i| children.item(i))
            .filter(|c| !c.has_attribute("data-clone"))
            .collect()
    }

    fn card_step(track: &Element) -> f64 {
        let Some(first) = real_cards(track).into_iter().next() else { return 0.0 };
        let gap = window()
            .get_computed_style(track)
            .ok()
            .flatten()
            .and_then(|s| s.get_property_value("column-gap").ok())
            .and_then(|g| g.trim_end_matches("px").parse::<f64>().ok())
            .unwrap_or(0.0);
        first.get_bounding_client_rect().width() + gap
    }

    fn style(track: &Element) -> Option<web_sys::CssStyleDeclaration> {
        track.clone().dyn_into::<HtmlElement>().ok().map(|t| t.style())
    }

    fn rebuild_reviews(track: &Element, cards: &[Element]) {
        let parent = cards_parent(track);
        if let Ok(clones) = parent.query_selector_all("[data-clone]") {
            for i in 0..clones.length() {
                if let Some(c) = clones.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                    c.remove();
                }
            }
        }
        for card in cards {
            let _ = parent.append_child(card);
        }
        let clone = |card: &Element| {
            let copy = card.clone_node_with_deep(true).ok()?.dyn_into::<Element>().ok()?;
            let _ = copy.set_attribute("data-clone", "");
            let _ = copy.set_attribute("aria-hidden", "true");
            Some(copy)
        };
        if let (Some(first), Some(last)) = (cards.first().and_then(clone), cards.last().and_then(clone)) {
            let _ = parent.insert_before(&last, parent.first_child().as_ref());
            let _ = parent.append_child(&first);
        }
        // Back to rest (showing the first real card) without animating.
        if let Some(style) = style(track) {
            let _ = style.set_property("transition", "none");
            let _ = style.set_property("transform", &format!("translateX(-{}px)", card_step(track)));
            let _ = track.get_bounding_client_rect();
            let _ = style.set_property("transition", "");
        }
    }

    pub fn setup_reviews(track: &Element) {
        let cards = real_cards(track);
        if cards.len() >= 2 {
            rebuild_reviews(track, &cards);
        }
    }

    pub fn slide_reviews(track: &Element, forward: bool) {
        let mut cards = real_cards(track);
        if cards.len() < 2 || track.has_attribute("data-animating") {
            return;
        }
        let _ = track.set_attribute("data-animating", "");
        if let Some(style) = style(track) {
            let _ = style.set_property("transition", "transform 0.4s ease");
            let offset = if forward { format!("translateX(-{}px)", 2.0 * card_step(track)) } else { "translateX(0)".to_owned() };
            let _ = style.set_property("transform", &offset);
        }
        let track_after = track.clone();
        let done = wasm_bindgen::closure::Closure::once_into_js(move || {
            if forward {
                cards.rotate_left(1);
            } else {
                cards.rotate_right(1);
            }
            rebuild_reviews(&track_after, &cards);
            let _ = track_after.remove_attribute("data-animating");
        });
        let options = web_sys::AddEventListenerOptions::new();
        options.set_once(true);
        let _ = track.add_event_listener_with_callback_and_add_event_listener_options("transitionend", done.unchecked_ref(), &options);
    }

    /// Shrinks the font of `.fit-lines` headings just enough that each
    /// <br>-separated line fits without wrapping (line lengths differ per
    /// language and font). Below `data-fit-min` px the lines wrap instead.
    fn fit_lines() {
        let Ok(headings) = document().query_selector_all(".fit-lines") else { return };
        for i in 0..headings.length() {
            let Some(el) = headings.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) else { continue };
            let min: f64 = el.get_attribute("data-fit-min").and_then(|m| m.parse().ok()).unwrap_or(0.0);
            let style = el.style();
            let _ = style.set_property("contain", "inline-size");
            let _ = style.set_property("font-size", "");
            let _ = style.set_property("white-space", "nowrap");
            let available = f64::from(el.client_width());
            let needed = f64::from(el.scroll_width());
            if needed <= available || available <= 0.0 {
                continue;
            }
            let current = window()
                .get_computed_style(&el)
                .ok()
                .flatten()
                .and_then(|s| s.get_property_value("font-size").ok())
                .and_then(|f| f.trim_end_matches("px").parse::<f64>().ok())
                .unwrap_or(16.0);
            let mut size = current * available / needed;
            if size < min {
                let _ = style.set_property("white-space", "");
                continue;
            }
            let _ = style.set_property("font-size", &format!("{size}px"));
            while f64::from(el.scroll_width()) > available && size > 1.0 {
                size -= 0.5;
                let _ = style.set_property("font-size", &format!("{size}px"));
            }
        }
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

/// A slide show around server-rendered markup: one `.carousel-slide` at a
/// time, `.prev-btn`/`.next-btn` arrows, optional `.carousel-dot`s, and the
/// next slide every 15 seconds (restarted by every click).
#[island]
pub fn SlideShow(children: Children) -> impl IntoView {
    let root = NodeRef::<leptos::html::Div>::new();
    #[cfg(feature = "hydrate")]
    {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        let current = Arc::new(AtomicUsize::new(0));
        let timer = StoredValue::new(None::<IntervalHandle>);
        let show = {
            let current = current.clone();
            move |index: usize| {
                let Some(root) = root.get_untracked() else { return };
                let slides = browser::all(&root, ".carousel-slide");
                if slides.is_empty() {
                    return;
                }
                let index = index % slides.len();
                for (i, slide) in slides.iter().enumerate() {
                    let _ = slide.class_list().toggle_with_force("active", i == index);
                }
                for (i, dot) in browser::all(&root, ".carousel-dot").iter().enumerate() {
                    let _ = dot.class_list().toggle_with_force("active", i == index);
                }
                current.store(index, Ordering::Relaxed);
            }
        };
        let restart = {
            let (show, current) = (show.clone(), current.clone());
            move || {
                if let Some(handle) = timer.get_value() {
                    handle.clear();
                }
                let (show, current) = (show.clone(), current.clone());
                timer.set_value(
                    set_interval_with_handle(move || show(current.load(Ordering::Relaxed) + 1), std::time::Duration::from_secs(15)).ok(),
                );
            }
        };
        Effect::new({
            let restart = restart.clone();
            move |_| {
                if root.get().is_some() {
                    restart();
                }
            }
        });
        let on_click = move |event: leptos::ev::MouseEvent| {
            let Some(root) = root.get_untracked() else { return };
            let count = browser::all(&root, ".carousel-slide").len().max(1);
            let now = current.load(Ordering::Relaxed);
            if browser::closest_within(&event, &root, ".next-btn").is_some() {
                show(now + 1);
            } else if browser::closest_within(&event, &root, ".prev-btn").is_some() {
                show(now + count - 1);
            } else if let Some(dot) = browser::closest_within(&event, &root, ".carousel-dot") {
                let dots = browser::all(&root, ".carousel-dot");
                if let Some(i) = dots.iter().position(|d| *d == dot) {
                    show(i);
                }
            } else {
                return;
            }
            restart();
        };
        return view! { <div class="slide-show" node_ref=root on:click=on_click>{children()}</div> };
    }
    #[cfg(not(feature = "hydrate"))]
    view! { <div class="slide-show" node_ref=root>{children()}</div> }
}

/// The Google reviews strip: endless in both directions, sliding one card
/// at a time. A copy of the last card waits before the first and of the
/// first after the last, so the incoming card slides in instead of popping.
#[island]
pub fn ReviewsCarousel(previous: String, next: String, children: Children) -> impl IntoView {
    let track = NodeRef::<leptos::html::Div>::new();
    #[allow(unused_variables)]
    let slide = move |forward: bool| {
        #[cfg(feature = "hydrate")]
        if let Some(track) = track.get_untracked() {
            browser::slide_reviews(&track, forward);
        }
    };
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if let Some(track) = track.get() {
            browser::setup_reviews(&track);
        }
    });
    view! {
        <div class="google-reviews-carousel">
            <button class="carousel-btn prev-btn" id="googleReviewsPrev" aria-label=previous on:click=move |_| slide(false)>
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"></polyline></svg>
            </button>
            <div class="google-reviews-viewport">
                <div class="google-reviews-grid" id="googleReviewsTrack" node_ref=track>{children()}</div>
            </div>
            <button class="carousel-btn next-btn" id="googleReviewsNext" aria-label=next on:click=move |_| slide(true)>
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>
            </button>
        </div>
    }
}
