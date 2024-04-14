use leptos::{component, CollectView, ReadSignal};
use leptos::{view, IntoView};

use crate::models::conversation::Conversation;

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
                        let message_class = if msg.is_human {
                            USER_MESSAGE_CLASS
                        } else {
                            ASSISTANT_MESSAGE_CLASS
                        };
                        view! {
                            <div class=format!(
                                "{COMMON_STYLE} {message_class}",
                            )>{msg.text.clone()}</div>
                        }
                    })
                    .collect_view()
            }}

        </div>
    }
}
