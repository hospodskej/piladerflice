//! The pages. Each loads its data through a server function, so the same
//! component renders on the server and the islands in it get plain props.

pub mod cart;
pub mod checkout;
pub mod eshop;
pub mod home;
pub mod info;
pub mod kontakt;
pub mod not_found;
pub mod sortiment;

use crate::i18n::{t, Locale};
use crate::visitor::visitor;
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use serde::{Deserialize, Serialize};

/// HTML that's safe to put into the page as is: admin texts after the
/// allow-list (see [`SafeHtml::from_admin`]) or translations marked `_html`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeHtml(String);

impl SafeHtml {
    /// Text typed in the admin, keeping only simple formatting (line breaks,
    /// bold, italics, m<sup>3</sup>) and never scripts or attributes.
    #[cfg(feature = "ssr")]
    pub fn from_admin(text: &str) -> SafeHtml {
        SafeHtml(
            ammonia::Builder::empty()
                .add_tags(["sup", "sub", "br", "strong", "em"])
                .clean(text)
                .to_string(),
        )
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<codon::Html> for SafeHtml {
    fn from(html: codon::Html) -> SafeHtml {
        SafeHtml(html.into_string())
    }
}

/// Renders `render(data)` once the server function `load` has the data, or
/// an error page when it fails.
pub fn loaded<T, Fut, V>(load: Fut, render: impl Fn(T) -> V + Send + Sync + 'static) -> impl IntoView
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
    Fut: std::future::Future<Output = Result<T, ServerFnError>> + Send + 'static,
    V: IntoView + 'static,
{
    let data = OnceResource::new_blocking(load);
    let render = std::sync::Arc::new(render);
    view! {
        <Suspense>
            {move || {
                let render = render.clone();
                Suspend::new(async move {
                    match data.await {
                        Ok(data) => render(data).into_any(),
                        Err(e) => view! { <ServerError error=e.to_string()/> }.into_any(),
                    }
                })
            }}
        </Suspense>
    }
}

#[component]
fn ServerError(error: String) -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        tracing::error!("page failed: {error}");
        if let Some(response) = use_context::<leptos_axum::ResponseOptions>() {
            response.set_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    let _ = error;
    let l = visitor().locale;
    view! {
        <main class="main not-found-section">
            <h1>{t::not_found::heading(l)}</h1>
        </main>
    }
}

/// The page's `<title>` and description, plus (for pages with a description:
/// the public ones) the canonical address and its other-language version.
#[component]
pub fn PageMeta(
    title: String,
    #[prop(optional, into)] description: Option<String>,
    /// The title without " | Pila Derflice" (the home page).
    #[prop(optional)]
    brand_only: bool,
    /// Query parameters that change what the page shows ("category").
    #[prop(optional)]
    keep_query: Vec<&'static str>,
) -> impl IntoView {
    let v = visitor();
    let l = v.locale;
    let title = if brand_only { title } else { format!("{title} | {}", t::site::name(l)) };
    let description = description.map(|d| truncate_words(&squish(&d), 160));
    let base = base_url();
    let path = crate::i18n::strip_prefix(&v.path).into_owned();
    let query: Vec<(String, String)> = v
        .query
        .as_deref()
        .unwrap_or_default()
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .filter(|(k, _)| keep_query.contains(k))
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect();
    let suffix = if query.is_empty() {
        String::new()
    } else {
        format!("?{}", query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&"))
    };
    let url = move |locale: Locale| format!("{base}{}{suffix}", locale.path(&path));
    let indexable = description.is_some();
    let og_url = if indexable { url(l) } else { format!("{}{}", base_url(), v.path) };
    let share_image = format!("{}/img/share/{}.webp", base_url(), l.as_str());

    view! {
        <Title text=title.clone()/>
        {description.clone().map(|d| view! { <Meta name="description" content=d/> })}
        {indexable.then(|| view! {
            <Link rel="canonical" href=url(l)/>
            <Link rel="alternate" hreflang=Locale::Cs.hreflang() href=url(Locale::Cs)/>
            <Link rel="alternate" hreflang=Locale::De.hreflang() href=url(Locale::De)/>
            <Link rel="alternate" hreflang="x-default" href=url(Locale::Cs)/>
        })}
        <Meta property="og:type" content="website"/>
        <Meta property="og:site_name" content=t::site::name(l)/>
        <Meta property="og:title" content=title/>
        <Meta property="og:url" content=og_url/>
        <Meta property="og:locale" content=l.og_locale()/>
        <Meta property="og:locale:alternate" content=l.other().og_locale()/>
        {description.map(|d| view! { <Meta property="og:description" content=d/> })}
        <Meta property="og:image" content=share_image/>
        <Meta property="og:image:type" content="image/webp"/>
        <Meta property="og:image:width" content="1200"/>
        <Meta property="og:image:height" content="630"/>
        <Meta property="og:image:alt" content=t::site::name(l)/>
        <Meta name="twitter:card" content="summary_large_image"/>
    }
}

/// The site's own address ("https://piladerflice.cz"), from `APP_HOST`.
pub fn base_url() -> String {
    #[cfg(feature = "ssr")]
    {
        if let Ok(host) = std::env::var("APP_HOST") {
            if !host.is_empty() {
                return format!("https://{host}");
            }
        }
        if let Some(parts) = use_context::<axum::http::request::Parts>() {
            if let Some(host) = parts.headers.get(axum::http::header::HOST).and_then(|h| h.to_str().ok()) {
                return format!("http://{host}");
            }
        }
    }
    String::new()
}

fn squish(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// At most `max` characters, cut at a space, with "..." when cut (as Rails' truncate).
fn truncate_words(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let room: String = text.chars().take(max - 3).collect();
    let cut = room.rfind(' ').map_or(room.as_str(), |i| &room[..i]);
    format!("{cut}...")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_at_a_word() {
        assert_eq!(truncate_words("short", 160), "short");
        assert_eq!(truncate_words("aaa bbb ccc", 9), "aaa...");
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn admin_html_keeps_only_simple_formatting() {
        let html = SafeHtml::from_admin(r#"9 800 Kč / m<sup>3</sup><br><script>x()</script><a href="x" onclick="y">link</a> <strong class="c">tučně</strong>"#);
        assert_eq!(html.as_str(), "9 800 Kč / m<sup>3</sup><br>link <strong>tučně</strong>");
    }
}
