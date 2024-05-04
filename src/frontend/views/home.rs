use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::*;
use leptos_router::{use_navigate, NavigateOptions};
use leptos_use::{use_websocket, UseWebsocketReturn};
use uuid::Uuid;

use crate::app::AppContext;
use crate::frontend::components::{Messages, Prompt};
use crate::models;
use crate::server::api::AskAssistant;

#[component]
pub(crate) fn Home() -> impl IntoView {
    // SAFETY: it's safe to unwrap because I provide context in App
    let AppContext {
        conversations,
        model,
    } = use_context().unwrap();

    let UseWebsocketReturn {
        message: assistant_message,
        send,
        ..
    } = use_websocket("ws://localhost:3000/ws");

    let navigate = use_navigate();

    let conversation_id = create_memo(|_| Uuid::new_v4());

    let user_prompt = create_rw_signal(String::new());
    let messages = create_rw_signal(Vec::<models::Message>::new());

    let send_user_prompt = create_server_action::<AskAssistant>();

    let dispatch = move || {
        let user_message = models::Message::user(user_prompt.get(), conversation_id());
        if user_message.content != "" {
            send(&serde_json::to_string(&user_message).unwrap());
            messages.update(|msgs| msgs.push(user_message.clone()));

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
        // dispatch();
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
            navigate(
                &format!("/c/{}", conversation_id.get()),
                NavigateOptions::default(),
            );
        };
    });

    create_effect(move |_| {
        logging::log!("message from ws: {:?}", assistant_message.get());
        if let Some(msg) = assistant_message.get() {
            let assistant_message: models::Message = serde_json::from_str(&msg).unwrap();

            if let Some(last_msg) = messages.get().last() {
                match models::Role::from(last_msg.role.clone()) {
                    models::Role::System => panic!("cannot happen"),
                    models::Role::User => {
                        logging::log!("adding assistant to the list: {:?}", assistant_message);
                        messages.update(|msgs| msgs.push(assistant_message));
                    }
                    models::Role::Assistant => {
                        // TODO: less sophisticated modify last message
                        // messages.update(|msgs| {
                        //     let last_index = msgs.len() - 1;
                        //     let mut updated_message = last_msg.clone();
                        //     updated_message.content.push_str(&assistant_message.content);

                        //     let _ = std::mem::replace(&mut msgs[last_index], updated_message);
                        // });

                        // TODO: more sophisticated update when backend reply with same message id
                        if let Some(message_to_update_position) = messages
                            .get()
                            .iter()
                            .position(|m| m.id == assistant_message.id)
                        {
                            logging::log!(
                                "adding assistant to the existing message at [{:?}]: {:?}",
                                message_to_update_position,
                                assistant_message
                            );
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
                    on_click=on_click
                    on_keydown=on_keydown
                    send_user_prompt=send_user_prompt
                />
            </div>
        </div>
    }
}
