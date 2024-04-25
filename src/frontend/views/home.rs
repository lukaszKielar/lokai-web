use leptos::*;

use crate::frontend::components::{Conversation, Sidebar};

// TODO: create sidebar once and pass it as a context in the app
#[component]
pub(crate) fn Home() -> impl IntoView {
    view! {
        <div class="overflow-hidden w-full h-screen relative flex">
            <div class="dark hidden flex-shrink-0 bg-gray-900 md:flex md:w-[260px] md:flex-col">
                <div class="flex h-full min-h-0 flex-col ">
                    <Sidebar/>
                </div>
            </div>
        // TODO: create NewConversation view
        // <Conversation/>
        </div>
    }
}
