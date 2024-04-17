use std::str::FromStr;

use crate::api::{chat, create_conversation};
use crate::components::conversation_area::ConversationArea;
use crate::components::prompt_area::PromptArea;
use crate::models::{Conversation, Message};

use leptos::*;
use leptos_meta::*;
use uuid::Uuid;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    logging::log!("loading");

    // TODO: read conversation from DB
    // TODO: list of conversations should be loaded from the server in reaction to changes
    // TODO: separately read conversation and messages
    let conversation_db = create_resource(
        || (),
        |_| async move { create_conversation().await.unwrap() },
    );
    let conversation_id = conversation_db.get().unwrap().id;
    let (conversation, set_conversation) = create_signal(conversation_db.get().unwrap());

    // TODO: throw an error when prompt is empty
    let send_prompt = create_action(move |prompt: &String| {
        logging::log!("preparing to send a message");
        let user_message = Message::user(prompt.to_owned(), conversation_id);
        let user_message_content = user_message.clone().content;
        set_conversation.update(move |c| c.push_user_message(user_message_content));
        chat(conversation_id, user_message)
    });

    // TODO: disable submit button when we're waiting for server's response
    create_effect(move |_| {
        if let Some(Ok(assistant_response)) = send_prompt.value().get() {
            set_conversation.update(move |c| c.push_assistant_message(assistant_response.content));
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
