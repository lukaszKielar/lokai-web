use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::frontend::components::{Conversation, Sidebar};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Title text="Welcome to LokAI!"/>
        <Stylesheet id="leptos" href="/pkg/lokai.css"/>
        <Meta charset="UTF-8"/>

        <main class="overflow-hidden w-full h-screen relative flex">
            <div class="dark hidden flex-shrink-0 bg-gray-900 md:flex md:w-[260px] md:flex-col">
                <div class="flex h-full min-h-0 flex-col ">
                    <Sidebar/>
                </div>
            </div>
            <Conversation/>
        </main>
    }
}
