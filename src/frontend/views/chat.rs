use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::html::*;
use leptos::*;
use leptos_router::use_params_map;
use leptos_use::{use_websocket, UseWebsocketReturn};
use uuid::Uuid;

use crate::app::AppContext;
use crate::frontend::components::{Messages, Prompt};
use crate::models;
use crate::server::api::get_conversation_messages;

#[component]
pub(crate) fn Chat() -> impl IntoView {
    // SAFETY: it's safe to unwrap because I provide context in App
    let AppContext {
        conversations: _,
        model,
    } = use_context().unwrap();

    let UseWebsocketReturn {
        ready_state,
        message: assistant_message,
        send,
        ..
    } = use_websocket("ws://localhost:3000/ws");
    // FIXME: it's ugly, but cannot use common `dispatch` without a clone
    let send_clone = send.clone();

    let params = use_params_map();
    let conversation_id = move || {
        params
            .get()
            .get("id")
            .map(|p| p.parse::<Uuid>().unwrap())
            .unwrap()
    };

    let server_response_pending = create_rw_signal(false);
    let user_prompt = create_rw_signal(String::new());
    let messages = create_rw_signal(Vec::<models::Message>::new());

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

    let dispatch = move |send: &dyn Fn(&str)| {
        let user_message =
            models::Message::user(Uuid::new_v4(), user_prompt.get(), conversation_id());
        if user_message.content != "" && !server_response_pending.get() {
            messages.update(|msgs| msgs.push(user_message.clone()));
            server_response_pending.set(true);
            send(&serde_json::to_string(&user_message).unwrap());
            logging::log!("prompt send to the server: {:?}", user_message.content);

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
            }

            if let Some(last_msg) = messages.get().last() {
                match models::Role::from(last_msg.role.clone()) {
                    models::Role::System => panic!("cannot happen"),
                    models::Role::User => {
                        if assistant_message.content != "" {
                            messages.update(|msgs| msgs.push(assistant_message));
                        }
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
                                <Transition>
                                    // TODO: reload only when necessary
                                    {if let Some(msgs) = db_messages.get() {
                                        messages.set(msgs);
                                    }}

                                </Transition>
                                // TODO: use For and properly update only necessary elements
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
