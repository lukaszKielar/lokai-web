use std::str::FromStr;

use crate::models::{Message, Role};
use crate::server::api::{get_conversation_messages, AskAssistant};

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

    let send_user_prompt = create_server_action::<AskAssistant>();
    let assistant_response = send_user_prompt.value();
    let messages = create_resource(assistant_response, |_| async {
        get_conversation_messages(Uuid::from_str("1ec2aa50-b36d-4bf6-a9d8-ef5da43425bb").unwrap())
            .await
            .unwrap()
    });
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

    let (user_prompt, set_user_prompt) = create_signal(String::new());

    view! {
        <main>
            <Stylesheet id="leptos" href="/pkg/lokai.css"/>

            <Title text="Welcome to LokAI!"/>

            <Meta charset="UTF-8"/>
            <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

            <div class="flex flex-col h-screen bg-gray-50 place-items-center">
                <div class="flex-1 w-[720px] bg-gray-100">
                    <Transition fallback=move || view! { <p>"Loading initial data..."</p> }>
                        <div class="flex flex-col space-y-2 p-2">{messages_view}</div>
                    </Transition>
                </div>
                <div class="flex-none h-20 w-[720px] bottom-0 place-items-center justify-center items-center bg-gray-200">
                    <form on:submit=move |ev: SubmitEvent| {
                        ev.prevent_default();
                        let user_message = Message::user(user_prompt(), conversation_id);
                        if user_message.content != "" {
                            send_user_prompt.dispatch(AskAssistant { user_message });
                            set_user_prompt("".to_string());
                        }
                    }>

                        <div class="flex items-center p-2">
                            <input
                                on:input=move |ev| {
                                    set_user_prompt(event_target_value(&ev));
                                }

                                class="border border-gray-300 rounded-lg px-4 py-2 w-full"
                                type="text"
                                placeholder="Message assistant..."
                                prop:value=user_prompt
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
