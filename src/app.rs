use std::str::FromStr;

use crate::components::conversation_area::ConversationArea;
use crate::components::prompt_area::PromptArea;
use crate::models::{Conversation, Message};
use crate::server::api::{chat, get_conversation_messages, CreateMessage};

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
    // let create_message = create_server_action::<create_message>();
    let messages = create_resource(
        || (),
        |_| {
            let conversation_id = Uuid::from_str("1ec2aa50-b36d-4bf6-a9d8-ef5da43425bb").unwrap();
            get_conversation_messages(conversation_id)
        },
    );
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

    view! {
        <main>
            <Stylesheet id="leptos" href="/pkg/lokai.css"/>

            // sets the document title
            <Title text="Welcome to LokAI!"/>

            <Meta charset="UTF-8"/>
            <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

            <div class="flex flex-col h-screen bg-gray-50 place-items-center">
                <div class="flex-1 w-[720px] bg-gray-100">
                    <ConversationArea conversation/>
                </div>
                <div class="flex-none h-20 w-[720px] bottom-0 place-items-center justify-center items-center bg-gray-200">
                    <PromptArea send_prompt/>
                </div>
            </div>
        </main>
    }
}
