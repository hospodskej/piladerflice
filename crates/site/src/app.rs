use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path, SsrMode,
};

/// The HTML document around every page.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="cs">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width,initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options islands=true/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Stylesheet id="site" href=crate::assets::STYLESHEET_HREF/>
        <Title text="Pila Derflice"/>
        <Router>
            <Routes fallback=|| "404">
                <Route path=path!("") view=Home ssr=SsrMode::Async/>
            </Routes>
        </Router>
    }
}

#[component]
fn Home() -> impl IntoView {
    view! {
        <div class="page-wrapper">
            <h1>"Pila Derflice"</h1>
            <Counter/>
        </div>
    }
}

#[island]
fn Counter() -> impl IntoView {
    let count = RwSignal::new(0);
    view! { <button class="btn-primary" on:click=move |_| *count.write() += 1>{count}</button> }
}
