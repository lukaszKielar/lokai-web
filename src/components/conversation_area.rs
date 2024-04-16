use leptos::{component, CollectView, ReadSignal};
use leptos::{view, IntoView};

use crate::models::{Conversation, Role};

const COMMON_STYLE: &str = "max-w-md p-4 mb-5 rounded-lg";
const USER_MESSAGE_CLASS: &str = "self-end bg-blue-500 text-white";
const ASSISTANT_MESSAGE_CLASS: &str = "self-start bg-green-500 text-white";

#[component]
pub fn ConversationArea(conversation: ReadSignal<Conversation>) -> impl IntoView {
    view! {
        <div class="flex flex-col space-y-2 p-2">
            {move || {
                conversation()
                    .iter()
                    .map(move |msg| {
                        let message_class = match Role::from(msg.role.as_ref()) {
                            Role::User => USER_MESSAGE_CLASS,
                            Role::Assistant => ASSISTANT_MESSAGE_CLASS,
                            _ => panic!("system message not supported yet"),
                        };
                        view! {
                            // TODO: define sytstem message class
                            // TODO: define trait for Tailwind, and implement it for Message struct
                            <div class=format!(
                                "{COMMON_STYLE} {message_class}",
                            )>{msg.content.clone()}</div>
                        }
                    })
                    .collect_view()
            }}

        </div>
    }
}
