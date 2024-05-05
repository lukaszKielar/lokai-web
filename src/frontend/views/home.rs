use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::html::*;
use leptos::*;
use leptos_router::{use_navigate, NavigateOptions};
use leptos_use::{use_websocket, UseWebsocketReturn};
use uuid::Uuid;

use crate::app::AppContext;
use crate::frontend::components::{Messages, Prompt};
use crate::models;

#[component]
pub(crate) fn Home() -> impl IntoView {
    // SAFETY: it's safe to unwrap because I provide context in App
    let AppContext {
        conversations,
        model,
    } = use_context().unwrap();

    let UseWebsocketReturn {
        ready_state,
        message: assistant_message,
        send,
        ..
    } = use_websocket("ws://localhost:3000/ws");
    // it's ugly, but send doesn't implment Copy
    let send_clone = send.clone();

    let navigate = use_navigate();

    let conversation_id = create_memo(|_| Uuid::new_v4());

    let server_response_pending = create_rw_signal(false);
    let user_prompt = create_rw_signal(String::new());
    let messages = create_rw_signal(Vec::<models::Message>::new());

    let bottom_of_chat_div = create_node_ref::<Div>();
    create_effect(move |_| {
        let _ = messages.get();
        if let Some(div) = bottom_of_chat_div.get() {
            // TODO: I need to scroll with options
            // https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollIntoView
            div.scroll_into_view();
        }
    });

    let dispatch = move |send: &dyn Fn(&str)| {
        let user_message =
            models::Message::user(Uuid::new_v4(), user_prompt.get(), conversation_id());
        if user_message.content != "" && !server_response_pending.get() {
            messages.update(|msgs| msgs.push(user_message.clone()));
            server_response_pending.set(true);
            send(&serde_json::to_string(&user_message).unwrap());
            logging::log!("prompt send to the server: {:?}", user_message.content);

            // TODO: I should prob get this object from server response
            let conversation = models::Conversation {
                id: conversation_id(),
                name: user_prompt.get(),
            };
            conversations.update(|convs| convs.push(conversation));
            user_prompt.set("".to_string());
        }
    };

    let on_click = move |ev: MouseEvent| {
        ev.prevent_default();
        dispatch(&send);
    };
    let on_keydown = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" && !ev.shift_key() {
            ev.prevent_default();
            dispatch(&send_clone);
        }
    };

    create_effect(move |_| {
        if let Some(msg) = assistant_message.get() {
            let assistant_message: models::Message = serde_json::from_str(&msg).unwrap();

            if assistant_message.content == "" {
                server_response_pending.set(false);
                navigate(
                    &format!("/c/{}", conversation_id.get()),
                    NavigateOptions::default(),
                );
            }

            if let Some(last_msg) = messages.get().last() {
                match models::Role::from(last_msg.role.clone()) {
                    models::Role::System => panic!("cannot happen"),
                    models::Role::User => {
                        messages.update(|msgs| msgs.push(assistant_message));
                    }
                    models::Role::Assistant => {
                        if let Some(message_to_update_position) = messages
                            .get()
                            .iter()
                            .position(|m| m.id == assistant_message.id)
                        {
                            // SAFETY: it's safe to unwrap, because I've just got the index
                            let current_message = messages
                                .get()
                                .get(message_to_update_position)
                                .unwrap()
                                .to_owned();
                            let mut updated_message = current_message.clone();
                            updated_message.content.push_str(&assistant_message.content);

                            messages.update(|msgs| {
                                let _ = std::mem::replace(
                                    &mut msgs[message_to_update_position],
                                    updated_message,
                                );
                            })
                        }
                    }
                }
            }
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
                                    "Model: " <b>{model}</b> ", Server: "
                                    {move || ready_state.get().to_string()}
                                </div>
                                <Messages messages=messages.into()/>
                                <div class="w-full h-32 flex-shrink-0"></div>
                                <div node_ref=bottom_of_chat_div></div>
                            </div>
                            <div class="flex flex-col items-center text-sm dark:bg-gray-800"></div>
                        </div>
                    </div>
                </div>
                <Prompt
                    user_prompt=user_prompt
                    on_click=on_click
                    on_keydown=on_keydown
                    server_response_pending=server_response_pending.read_only()
                />
            </div>
        </div>
    }
}
