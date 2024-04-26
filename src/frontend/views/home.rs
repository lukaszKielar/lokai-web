use leptos::*;
use uuid::Uuid;

use crate::app::{MessagesContext, SettingsContext};
use crate::frontend::components::{Messages, Prompt};

// TODO: Make it similar to Chat area
#[component]
pub(crate) fn Home() -> impl IntoView {
    let conversation_id = create_memo(|_| Uuid::new_v4());

    // SAFETY: it's safe to unwrap because I provide context in App
    let MessagesContext {
        messages,
        set_messages,
    } = use_context().unwrap();
    // SAFETY: it's safe to unwrap because I provide context in App
    let SettingsContext { model } = use_context().unwrap();

    view! {
        {set_messages(Vec::new())}
        <div class="flex max-w-full flex-1 flex-col">
            <div class="relative h-full w-full transition-width flex flex-col overflow-hidden items-stretch flex-1">
                <div class="flex-1 overflow-hidden">
                    <div class="scroll-to-bottom--css-ikyem-79elbk h-full dark:bg-gray-800">
                        <div class="scroll-to-bottom--css-ikyem-1n7m0yu">
                            <div class="flex flex-col items-center text-sm bg-gray-800">

                                <div class="flex w-full items-center justify-center gap-1 border-b border-black/10 bg-gray-50 p-3 text-gray-500 dark:border-gray-900/50 dark:bg-gray-700 dark:text-gray-300">
                                    "Model: " <b>{model}</b>
                                </div>
                                <Messages messages=messages.into()/>
                            </div>
                            <div class="flex flex-col items-center text-sm dark:bg-gray-800"></div>
                        </div>
                    </div>
                </div>
                <Prompt
                    conversation_id=MaybeSignal::derive(conversation_id)
                    set_messages=set_messages
                />
            </div>
        </div>
    }
}
