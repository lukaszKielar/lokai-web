use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::frontend::views::{Home, NotFound};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Title text="Welcome to LokAI!"/>
        <Stylesheet id="leptos" href="/pkg/lokai.css"/>
        <Meta charset="UTF-8"/>
        <Router>
            <main>
                <Routes>
                    <Route path="/" view=Home/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}
