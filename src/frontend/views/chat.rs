use leptos::{html::Div, *};
use leptos_router::use_params_map;
use uuid::Uuid;

use crate::app::{MessagesContext, SettingsContext};
use crate::frontend::components::{Messages, Prompt};
use crate::server::api::get_conversation_messages;

#[component]
pub(crate) fn Chat() -> impl IntoView {
    let params = use_params_map();
    let conversation_id = move || {
        params
            .get()
            .get("id")
            .map(|p| p.parse::<Uuid>().unwrap())
            .unwrap()
    };

    let db_messages = create_resource(
        move || conversation_id(),
        move |id| async move { get_conversation_messages(id).await.unwrap() },
    );

    // SAFETY: it's safe to unwrap because I provide context in App
    let MessagesContext {
        messages,
        set_messages,
    } = use_context::<MessagesContext>().unwrap();
    // SAFETY: it's safe to unwrap because I provide context in App
    let SettingsContext { model } = use_context().unwrap();

    let bottom_of_chat_div = create_node_ref::<Div>();
    create_effect(move |_| {
        let _ = messages();
        if let Some(div) = bottom_of_chat_div.get() {
            // TODO: I need to scroll with options
            // https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollIntoView
            div.scroll_into_view();
        }
    });

    view! {
        <div class="flex max-w-full flex-1 flex-col">
            <div class="relative h-full w-full transition-width flex flex-col overflow-hidden items-stretch flex-1">
                <div class="flex-1 overflow-hidden">
                    <div class="scroll-to-bottom--css-ikyem-79elbk h-full dark:bg-gray-800">
                        <div class="scroll-to-bottom--css-ikyem-1n7m0yu">
                            <div class="flex flex-col items-center text-sm bg-gray-800">
                                <Transition fallback=move || {
                                    view! {
                                        <div class="flex w-full items-center justify-center gap-1 border-b border-black/10 bg-gray-50 p-3 text-gray-500 dark:border-gray-900/50 dark:bg-gray-700 dark:text-gray-300">
                                            "Loading initial data..."
                                        </div>
                                    }
                                }>
                                    {if let Some(messages) = db_messages.get() {
                                        set_messages(messages);
                                    }}
                                    <div class="flex w-full items-center justify-center gap-1 border-b border-black/10 bg-gray-50 p-3 text-gray-500 dark:border-gray-900/50 dark:bg-gray-700 dark:text-gray-300">
                                        "Model: " <b>{model}</b>
                                    </div> <Messages messages=messages.into()/>
                                    <div class="w-full h-32 flex-shrink-0"></div>
                                    <div node_ref=bottom_of_chat_div></div>
                                </Transition>
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
