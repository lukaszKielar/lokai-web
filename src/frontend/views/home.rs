use leptos::*;

use crate::app::MessagesContext;

// TODO: Make it similar to Chat area
#[component]
pub(crate) fn Home() -> impl IntoView {
    // SAFETY: it's safe to unwrap because I provide context in App
    let MessagesContext {
        messages: _messages,
        set_messages,
    } = use_context::<MessagesContext>().unwrap();

    view! {
        {set_messages(Vec::new())}
        <div class="flex max-w-full dark:bg-gray-800">
            <div class="h-10">A</div>
            <div class="h-10">B</div>
            <div class="h-10">C</div>
        </div>
    }
}
