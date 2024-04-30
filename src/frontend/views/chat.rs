use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::{html::Div, *};
use leptos_router::use_params_map;
use uuid::Uuid;

use crate::app::AppContext;
use crate::frontend::components::{Messages, Prompt};
use crate::models;
use crate::server::api::{get_conversation_messages, AskAssistant};

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

    let user_prompt = create_rw_signal(String::new());
    let messages = create_rw_signal(Vec::<models::Message>::new());

    // SAFETY: it's safe to unwrap because I provide context in App
    let AppContext {
        conversations: _,
        model,
    } = use_context().unwrap();

    let db_messages = create_resource(
        move || conversation_id(),
        move |id| async move { get_conversation_messages(id).await.unwrap() },
    );

    let bottom_of_chat_div = create_node_ref::<Div>();
    create_effect(move |_| {
        let _ = messages.get();
        if let Some(div) = bottom_of_chat_div.get() {
            // TODO: I need to scroll with options
            // https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollIntoView
            div.scroll_into_view();
        }
    });

    let send_user_prompt = create_server_action::<AskAssistant>();

    let dispatch = move || {
        let user_message = models::Message::user(user_prompt.get(), conversation_id());
        let user_message_clone = user_message.clone();
        if user_message.content != "" {
            messages.update(|msgs| msgs.push(user_message_clone));
            send_user_prompt.dispatch(AskAssistant {
                user_message,
                new_conversation: false,
            });
            user_prompt.set("".to_string());
        }
    };

    let on_click = move |ev: MouseEvent| {
        ev.prevent_default();
        dispatch();
    };
    let on_keydown = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" && !ev.shift_key() {
            ev.prevent_default();
            dispatch();
        }
    };

    create_effect(move |_| {
        if let Some(response) = send_user_prompt.value().get() {
            // TODO: handle errors
            let assistant_response = response.unwrap();
            messages.update(|msgs| msgs.push(assistant_response));
        };
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
                                    // TODO: use For and properly update only necessary elements

                                    {if let Some(msgs) = db_messages.get() {
                                        messages.set(msgs);
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
                    user_prompt=user_prompt
                    on_click=on_click
                    on_keydown=on_keydown
                    send_user_prompt=send_user_prompt
                />
            </div>
        </div>
    }
}
