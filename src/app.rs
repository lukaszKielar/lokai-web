use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::frontend::components::Sidebar;
use crate::frontend::views::{Chat, Home};
use crate::{models, MODEL};

#[derive(Copy, Clone)]
pub struct AppContext {
    pub conversations: RwSignal<Vec<models::Conversation>>,
    pub model: ReadSignal<String>,
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let conversations = create_rw_signal(Vec::<models::Conversation>::new());
    let (model, _) = create_signal(String::from(MODEL));
    provide_context(AppContext {
        model,
        conversations,
    });

    view! {
        <Title text="Welcome to LokAI!"/>
        <Stylesheet id="leptos" href="/pkg/lokai.css"/>
        <Meta charset="UTF-8"/>
        <Router>
            <main>
                <div class="overflow-hidden w-full h-screen relative flex">
                    <div class="dark hidden flex-shrink-0 bg-gray-900 md:flex md:w-[260px] md:flex-col">
                        <div class="flex h-full min-h-0 flex-col ">
                            <Sidebar/>
                        </div>
                    </div>
                    <Routes>
                        <Route path="/" view=Home/>
                        <Route path="/c/:id" view=Chat/>
                    </Routes>
                </div>
            </main>
        </Router>
    }
}
