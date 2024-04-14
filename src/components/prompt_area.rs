use leptos::ev::SubmitEvent;
use leptos::{component, create_signal, event_target_value, view, Action, IntoView, ServerFnError};

use crate::models::Message;

// TODO: disable button when we wait for response
// TODO: make button grey while waiting for response
#[component]
pub fn PromptArea(send_prompt: Action<String, Result<Message, ServerFnError>>) -> impl IntoView {
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
    }
}
