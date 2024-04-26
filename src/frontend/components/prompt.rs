use icondata;
use leptos::ev::{Event, KeyboardEvent, MouseEvent};
use leptos::{html::Textarea, *};
use leptos_icons::Icon;
use uuid::Uuid;

use crate::models;
use crate::server::api::AskAssistant;

#[component]
fn PromptSubmitButton<F>(on_click: F, button_disabled: ReadSignal<bool>) -> impl IntoView
where
    F: Fn(MouseEvent) + 'static,
{
    view! {
        <button
            class="absolute p-1 rounded-md bottom-1.5 md:bottom-2.5 bg-transparent disabled:bg-gray-500 right-1 md:right-2 disabled:opacity-40"
            on:click=on_click
            disabled=button_disabled()
        >
            <Icon icon=icondata::LuSend class="h-4 w-4 mr-1 text-white "/>
        </button>
    }
}

#[component]
fn PromptTextArea<FIn, FKdn>(
    on_input: FIn,
    on_keydown: FKdn,
    #[prop(into)] user_prompt: Signal<String>,
    #[prop(default = 24)] min_height: i32,
    #[prop(default = 216)] max_height: i32,
) -> impl IntoView
where
    FIn: Fn(Event) + 'static,
    FKdn: Fn(KeyboardEvent) + 'static,
{
    let textarea = create_node_ref::<Textarea>();
    let (dynamic_height, set_dynamic_height) = create_signal(min_height);
    // automatically resize textarea
    create_effect(move |_| {
        set_dynamic_height(min_height);
        // reset if user_prompt() if empty
        let scroll_height = if user_prompt() == "" {
            min_height
        } else {
            // SAFETY: effect is triggered by user input into Textarea, so element has been already loaded,
            // so it's safe to unwrap NodeRef
            textarea.get().unwrap().scroll_height()
        };
        let style_height = std::cmp::min(scroll_height, max_height);
        set_dynamic_height(style_height)
    });

    view! {
        <textarea
            on:input=on_input
            on:keydown=on_keydown
            type="text"
            prop:value=user_prompt
            node_ref=textarea
            tab_index=0
            placeholder="Message LokAI..."
            class=move || {
                format!(
                    "m-0 w-full resize-none border-0 bg-transparent p-0 pr-7 focus:ring-0 focus-visible:ring-0 dark:bg-transparent pl-2 h-[{min_height}px] max-h-[{max_height}px] overflow-y-auto",
                )
            }

            style:height=move || { format!("{}px", dynamic_height()) }
        ></textarea>
    }
}

#[component]
pub(crate) fn Prompt(
    #[prop(default = 24)] min_textarea_height: i32,
    #[prop(default = 216)] max_textarea_height: i32,
    conversation_id: MaybeSignal<Uuid>,
    set_messages: WriteSignal<Vec<models::Message>>,
) -> impl IntoView {
    let (user_prompt, set_user_prompt) = create_signal(String::new());
    let (button_disabled, set_button_disabled) = create_signal(true);

    let send_user_prompt = create_server_action::<AskAssistant>();
    let assistant_response_value = send_user_prompt.value();
    create_effect(move |_| {
        if let Some(response) = assistant_response_value.get() {
            let assistant_response = response.unwrap();
            set_messages.update(|msgs| msgs.push(assistant_response));
        }
    });

    create_effect(move |_| {
        if user_prompt().len() == 0 || send_user_prompt.pending().get() {
            set_button_disabled(true)
        } else {
            set_button_disabled(false)
        };
    });

    let on_click = move |ev: MouseEvent| {
        ev.prevent_default();
        let user_message = models::Message::user(user_prompt(), conversation_id());
        let user_message_clone = user_message.clone();
        if user_message.content != "" {
            set_messages.update(|msgs| msgs.push(user_message_clone));
            send_user_prompt.dispatch(AskAssistant { user_message });
            set_user_prompt("".to_string());
        }
    };

    let on_input = move |ev| set_user_prompt(event_target_value(&ev));
    let on_keydown = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" && !ev.shift_key() {
            ev.prevent_default();
            let user_message = models::Message::user(user_prompt(), conversation_id());
            let user_message_clone = user_message.clone();
            if user_message.content != "" {
                set_messages.update(|msgs| msgs.push(user_message_clone));
                send_user_prompt.dispatch(AskAssistant { user_message });
                set_user_prompt("".to_string());
            }
        }
    };

    view! {
        <div class="absolute bottom-0 left-0 w-full border-t md:border-t-0 dark:border-white/20 md:border-transparent md:dark:border-transparent md:bg-vert-light-gradient bg-white dark:bg-gray-800 md:!bg-transparent dark:md:bg-vert-dark-gradient pt-2">
            <form class="stretch mx-2 flex flex-row gap-3 last:mb-2 md:mx-4 md:last:mb-6 lg:mx-auto lg:max-w-2xl xl:max-w-3xl">

                <div class="relative flex flex-col h-full flex-1 items-stretch md:flex-col">
                    <div class="flex flex-col w-full py-2 flex-grow md:py-3 md:pl-4 relative border border-black/10 bg-white dark:border-gray-900/50 dark:text-white dark:bg-gray-700 rounded-md shadow-[0_0_10px_rgba(0,0,0,0.10)] dark:shadow-[0_0_15px_rgba(0,0,0,0.10)]">
                        <PromptTextArea
                            on_input=on_input
                            on_keydown=on_keydown
                            min_height=min_textarea_height
                            max_height=max_textarea_height
                            user_prompt=user_prompt
                        />
                        <PromptSubmitButton on_click=on_click button_disabled=button_disabled/>
                    </div>
                </div>
            </form>
            <div class="px-3 pt-2 pb-3 text-center text-xs text-black/50 dark:text-white/50 md:px-4 md:pt-3 md:pb-6">
                <span>"Enjoy your self-hosted LokAI!"</span>
            </div>
        </div>
    }
}
