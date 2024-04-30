use leptos::ev::{Event, KeyboardEvent, MouseEvent};
use leptos::*;
use leptos_router::{use_navigate, NavigateOptions};
use uuid::Uuid;

use crate::app::AppContext;
use crate::frontend::components::{Messages, Prompt};
use crate::models;
use crate::server::api::AskAssistant;

#[component]
pub(crate) fn Home() -> impl IntoView {
    let navigate = use_navigate();

    let conversation_id = create_memo(|_| Uuid::new_v4());

    let (user_prompt, set_user_prompt) = create_signal(String::new());
    let messages = create_rw_signal(Vec::<models::Message>::new());

    // SAFETY: it's safe to unwrap because I provide context in App
    let AppContext {
        conversations,
        model,
    } = use_context().unwrap();

    let send_user_prompt = create_server_action::<AskAssistant>();

    let dispatch = move || {
        let user_message = models::Message::user(user_prompt(), conversation_id());
        let user_message_clone = user_message.clone();
        if user_message.content != "" {
            messages.update(|msgs| msgs.push(user_message_clone));
            send_user_prompt.dispatch(AskAssistant {
                user_message,
                new_conversation: true,
            });
            // TODO: I should prob get this object from server response
            let conversation = models::Conversation {
                id: conversation_id(),
                name: user_prompt(),
            };
            conversations.update(|convs| convs.push(conversation));
            set_user_prompt("".to_string());
        }
    };

    let on_click = move |ev: MouseEvent| {
        ev.prevent_default();
        dispatch();
    };
    let on_input = move |ev: Event| set_user_prompt(event_target_value(&ev));
    let on_keydown = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" && !ev.shift_key() {
            ev.prevent_default();
            dispatch();
        }
    };

    create_effect(move |_| {
        if let Some(_) = send_user_prompt.value().get() {
            navigate(
                &format!("/c/{}", conversation_id.get()),
                NavigateOptions::default(),
            );
        };
    });

    view! {
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
                    user_prompt=user_prompt
                    set_messages=messages.write_only()
                    on_input=on_input
                    on_click=on_click
                    on_keydown=on_keydown
                    send_user_prompt=send_user_prompt
                />
            </div>
        </div>
    }
}
