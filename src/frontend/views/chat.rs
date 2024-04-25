use leptos::*;

use crate::frontend::components::{Conversation, Sidebar};

#[component]
pub(crate) fn Chat() -> impl IntoView {
    view! {
        <div class="overflow-hidden w-full h-screen relative flex">
            <div class="dark hidden flex-shrink-0 bg-gray-900 md:flex md:w-[260px] md:flex-col">
                <div class="flex h-full min-h-0 flex-col ">
                    <Sidebar/>
                </div>
            </div>
            <Conversation/>
        </div>
    }
}
