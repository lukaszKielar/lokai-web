use crate::api::replai;
use crate::components::conversation_area::ConversationArea;
use crate::components::prompt_area::PromptArea;
use crate::models::conversation::Conversation;
use crate::models::message::Message;

use leptos::*;
use leptos_meta::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let (conversation, set_conversation) = create_signal(Conversation::new());

    // TODO: throw an error when prompt is empty
    let send_prompt = create_action(move |prompt: &String| {
        let prompt = prompt.to_owned();
        let human_msg = Message::human(prompt.clone());
        set_conversation.update(move |c| c.append_message(human_msg));

        replai(prompt, conversation().context)
    });

    create_effect(move |_| {
        if let Some(_) = send_prompt.input().get() {
            let assistant_message = Message::assistant("...".to_string());

            set_conversation.update(move |c| c.append_message(assistant_message))
        }
    });

    // TODO: disable submit button when we're waiting for server's response
    create_effect(move |_| {
        if let Some(Ok(assistant_response)) = send_prompt.value().get() {
            set_conversation.update(move |c| {
                c.modify_last_msg(assistant_response.response);
                c.context.replace(assistant_response.context);
            });
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
