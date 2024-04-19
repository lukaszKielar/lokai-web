use std::str::FromStr;

use crate::models::{Conversation, Message, Role};
use crate::server::api::{chat, get_conversation_messages, CreateMessage};

use leptos::ev::SubmitEvent;
use leptos::*;
use leptos_meta::*;
use uuid::Uuid;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // TODO: read conversation from DB
    // TODO: list of conversations should be loaded from the server in reaction to changes
    // TODO: separately read conversation and messages
    let conversation_id = Uuid::from_str("1ec2aa50-b36d-4bf6-a9d8-ef5da43425bb").unwrap();

    let create_message = create_server_action::<CreateMessage>();
    let messages = create_resource(
        || (),
        |_| async {
            get_conversation_messages(
                Uuid::from_str("1ec2aa50-b36d-4bf6-a9d8-ef5da43425bb").unwrap(),
            )
            .await
            .unwrap()
        },
    );

    let messages_view = move || {
        messages.get().map(|messages| {
            messages
                .iter()
                .map(|message| {
                    let message_class = match Role::from(message.role.as_ref()) {
                        Role::User => "self-end bg-blue-500 text-white",
                        Role::Assistant => "self-start bg-green-500 text-white",
                        _ => panic!("system message not supported yet"),
                    };
                    view! {
                        <div class=format!(
                            "max-w-md p-4 mb-5 rounded-lg {message_class}",
                        )>{message.content.clone()}</div>
                    }
                })
                .collect_view()
        })
    };
    let (conversation, set_conversation) = create_signal(Conversation::new(conversation_id));

    // TODO: throw an error when prompt is empty
    let send_prompt = create_action(move |prompt: &String| {
        let conversation_id = conversation().id;
        logging::log!("preparing to send a message");
        let user_message = Message::user(prompt.to_owned(), conversation_id);
        let user_message_clone = user_message.clone();
        set_conversation.update(move |c| c.push_message(user_message_clone));
        chat(user_message)
    });

    // TODO: disable submit button when we're waiting for server's response
    create_effect(move |_| {
        if let Some(Ok(assistant_response)) = send_prompt.value().get() {
            set_conversation.update(move |c| c.push_message(assistant_response));
        }
    });

    let (prompt, set_prompt) = create_signal(String::new());

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let value = prompt();
        if value != "" {
            send_prompt.dispatch(value);
            set_prompt("".to_string());
        }
    };

    view! {
        <main>
            <Stylesheet id="leptos" href="/pkg/lokai.css"/>

            <Title text="Welcome to LokAI!"/>

            <Meta charset="UTF-8"/>
            <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

            <div class="flex flex-col h-screen bg-gray-50 place-items-center">
                <div class="flex-1 w-[720px] bg-gray-100">
                    <Transition fallback=move || view! { <p>"Loading initial data..."</p> }>
                        <div class="flex flex-col space-y-2 p-2">
                            {messages_view}
                            {{
                                move || {
                                    conversation()
                                        .iter()
                                        .map(move |msg| {
                                            let message_class = match Role::from(msg.role.as_ref()) {
                                                Role::User => "self-end bg-blue-500 text-white",
                                                Role::Assistant => "self-start bg-green-500 text-white",
                                                _ => panic!("system message not supported yet"),
                                            };
                                            view! {
                                                // TODO: define sytstem message class
                                                // TODO: define trait for Tailwind, and implement it for Message struct
                                                <div class=format!(
                                                    "max-w-md p-4 mb-5 rounded-lg {message_class}",
                                                )>{msg.content.clone()}</div>
                                            }
                                        })
                                        .collect_view()
                                }
                            }}

                        </div>
                    </Transition>
                </div>
                <div class="flex-none h-20 w-[720px] bottom-0 place-items-center justify-center items-center bg-gray-200">
                    <form on:submit=on_submit>
                        <div class="flex items-center p-2">
                            <input
                                on:input=move |ev| {
                                    set_prompt(event_target_value(&ev));
                                }

                                class="border border-gray-300 rounded-lg px-4 py-2 w-full"
                                type="text"
                                placeholder="Message assistant..."
                                prop:value=prompt
                            />
                            <button
                                class="ml-2 bg-blue-500 hover:bg-blue-700 text-white px-4 py-2 rounded-lg"
                                type="submit"
                            >
                                Send
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </main>
    }
}
