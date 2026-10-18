//! Small helpers for the browser's page, used by the islands' event
//! handlers and effects (which only ever run in the browser).

use leptos::prelude::{document, window};
use wasm_bindgen::{JsCast, JsValue};

pub fn has_root_class(name: &str) -> bool {
    document()
        .document_element()
        .is_some_and(|root| root.class_list().contains(name))
}

/// Adds or removes a class of `<html>` (e.g. "dark-mode").
pub fn set_root_class(name: &str, on: bool) {
    if let Some(root) = document().document_element() {
        let _ = root.class_list().toggle_with_force(name, on);
    }
}

/// A cookie for the whole site, kept for `max_age` seconds.
pub fn set_cookie(name: &str, value: &str, max_age: u32) {
    let cookie = format!("{name}={value}; path=/; max-age={max_age}; SameSite=Lax");
    let _ = document().unchecked_into::<web_sys::HtmlDocument>().set_cookie(&cookie);
}

#[cfg(feature = "hydrate")]
pub fn cookie(name: &str) -> Option<String> {
    let cookies = document().unchecked_into::<web_sys::HtmlDocument>().cookie().ok()?;
    cookies
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.to_string())
}

/// A value from the browser's local storage.
#[cfg(feature = "hydrate")]
pub fn stored(key: &str) -> Option<String> {
    window().local_storage().ok()??.get_item(key).ok()?
}

/// "hidden" keeps the page from scrolling (under the mobile menu); "" lets it.
pub fn set_body_overflow(value: &str) {
    if let Some(body) = document().body() {
        let _ = body.style().set_property("overflow", value);
    }
}

pub fn scroll_y() -> f64 {
    window().scroll_y().unwrap_or(0.0)
}

/// The height of the site's header; 0 when there is none.
pub fn header_height() -> f64 {
    document()
        .query_selector(".site-header")
        .ok()
        .flatten()
        .and_then(|header| header.dyn_into::<web_sys::HtmlElement>().ok())
        .map_or(0.0, |header| f64::from(header.offset_height()))
}

/// "#cenik", or "" without one.
pub fn location_hash() -> String {
    window().location().hash().unwrap_or_default()
}

/// Tells Google Analytics (loaded by the page) whether it may count.
pub fn update_analytics_consent(granted: bool) {
    let Ok(gtag) = js_sys::Reflect::get(&window(), &JsValue::from_str("gtag")) else {
        return;
    };
    let Some(gtag) = gtag.dyn_ref::<js_sys::Function>() else {
        return;
    };
    let settings = js_sys::Object::new();
    let value = if granted { "granted" } else { "denied" };
    let _ = js_sys::Reflect::set(&settings, &"analytics_storage".into(), &value.into());
    let _ = gtag.call3(&JsValue::NULL, &"consent".into(), &"update".into(), &settings);
}

/// Makes each `<br>`-separated line of `element` fit on one line (see
/// `FitLines`), now, when its width changes and once the fonts are loaded.
pub fn fit_lines(element: web_sys::HtmlElement, min: f64) {
    use std::cell::Cell;
    use wasm_bindgen::closure::Closure;

    // Keeps the unwrapped lines from widening the parent (a grid column
    // sizes itself to its content).
    let _ = element.style().set_property("contain", "inline-size");
    let width = Cell::new(-1.0);
    let target = element.clone();
    let resized = Closure::<dyn FnMut(js_sys::Array)>::new(move |entries: js_sys::Array| {
        let entry: web_sys::ResizeObserverEntry = entries.get(0).unchecked_into();
        let now = entry.content_rect().width();
        if now != width.replace(now) {
            fit(&target, min);
        }
    });
    if let Ok(observer) = web_sys::ResizeObserver::new(resized.as_ref().unchecked_ref()) {
        observer.observe(&element);
    }
    // The observer lives as long as the page.
    resized.forget();
    let fonts_loaded = Closure::once(move |_: JsValue| fit(&element, min));
    if let Ok(ready) = document().fonts().ready() {
        let _ = ready.then(&fonts_loaded);
    }
    fonts_loaded.forget();
}

fn fit(element: &web_sys::HtmlElement, min: f64) {
    let style = element.style();
    let _ = style.set_property("font-size", "");
    let _ = style.set_property("white-space", "nowrap");
    let available = f64::from(element.client_width());
    let needed = f64::from(element.scroll_width());
    if needed <= available {
        return;
    }
    let current = window()
        .get_computed_style(element)
        .ok()
        .flatten()
        .and_then(|computed| computed.get_property_value("font-size").ok())
        .and_then(|size| size.trim_end_matches("px").parse::<f64>().ok())
        .unwrap_or(16.0);
    let mut size = current * available / needed;
    if size < min {
        let _ = style.set_property("white-space", "");
        return;
    }
    let _ = style.set_property("font-size", &format!("{size}px"));
    while f64::from(element.scroll_width()) > available && size > 1.0 {
        size -= 0.5;
        let _ = style.set_property("font-size", &format!("{size}px"));
    }
}

/// How far a row of cards moves for one card: its width and the gap.
pub fn card_step(track: &web_sys::HtmlElement) -> f64 {
    let Some(card) = track.first_element_child() else {
        return 0.0;
    };
    let gap = window()
        .get_computed_style(track)
        .ok()
        .flatten()
        .and_then(|style| style.get_property_value("column-gap").ok())
        .and_then(|gap| gap.trim_end_matches("px").parse::<f64>().ok())
        .unwrap_or(0.0);
    card.get_bounding_client_rect().width() + gap
}

/// Scrolls the page smoothly so that `element` is in the middle.
pub fn scroll_to_center(element: &web_sys::Element) {
    let options = web_sys::ScrollIntoViewOptions::new();
    options.set_behavior(web_sys::ScrollBehavior::Smooth);
    options.set_block(web_sys::ScrollLogicalPosition::Center);
    element.scroll_into_view_with_scroll_into_view_options(&options);
}

/// Reports an order to Google Analytics as a purchase, once per browser
/// tab (the thank-you page may be reloaded).
pub fn report_purchase(order_id: i64, total_czk: f64, items_json: &str) {
    let key = format!("ga_purchase_fired_{order_id}");
    if let Ok(Some(storage)) = window().session_storage() {
        if storage.get_item(&key).ok().flatten().is_some() {
            return;
        }
        let _ = storage.set_item(&key, "1");
    }
    let Ok(gtag) = js_sys::Reflect::get(&window(), &JsValue::from_str("gtag")) else {
        return;
    };
    let Some(gtag) = gtag.dyn_ref::<js_sys::Function>() else {
        return;
    };
    let event = js_sys::Object::new();
    let set = |name: &str, value: JsValue| {
        let _ = js_sys::Reflect::set(&event, &name.into(), &value);
    };
    set("transaction_id", order_id.to_string().into());
    set("value", total_czk.into());
    set("currency", "CZK".into());
    set(
        "items",
        js_sys::JSON::parse(items_json).unwrap_or_else(|_| js_sys::Array::new().into()),
    );
    let _ = gtag.call3(&JsValue::NULL, &"event".into(), &"purchase".into(), &event);
}
