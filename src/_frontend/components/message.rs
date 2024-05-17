use leptos::*;
use leptos_icons::Icon;

use crate::models::{Message as MessageModel, Role};

#[component]
pub(crate) fn Message(message: MaybeSignal<MessageModel>) -> impl IntoView {
    let message = message.get();
    let message_role = message.role;
    let message_content = message.content.into_view();

    let is_user = match Role::from(message_role) {
        Role::Assistant | Role::System => false,
        Role::User => true,
    };

    let message_class = {
        let message_role_class = match is_user {
            true => "dark:bg-gray-800",
            false => "bg-gray-50 dark:bg-[#444654]",
        };
        format!("group w-full text-gray-800 dark:text-gray-100 border-b border-black/10 dark:border-gray-900/50 {message_role_class}")
    };

    let message_icon = {
        let class = "h-4 w-4 text-white";
        match is_user {
            true => view! { <Icon icon=icondata::LuUser class=class/> },
            false => view! { <Icon icon=icondata::LuBot class=class/> },
        }
    };

    view! {
        <div class=message_class>
            <div class="text-base gap-4 md:gap-6 md:max-w-2xl lg:max-w-xl xl:max-w-3xl flex lg:px-0 m-auto w-full">
                <div class="flex flex-row gap-4 md:gap-6 md:max-w-2xl lg:max-w-xl xl:max-w-3xl p-4 md:py-6 lg:px-0 m-auto w-full">
                    <div class="w-8 flex flex-col relative items-end">
                        <div class="relative h-7 w-7 p-1 rounded-full text-white flex items-center justify-center bg-black/75 text-opacity-100r">
                            {message_icon}
                        </div>
                        <div class="text-xs flex items-center justify-center gap-1 absolute left-0 top-2 -ml-4 -translate-x-full group-hover:visible !invisible">
                            <button disabled class="text-gray-300 dark:text-gray-400"></button>
                            <span class="flex-grow flex-shrink-0">1 / 1</span>
                            <button disabled class="text-gray-300 dark:text-gray-400"></button>
                        </div>
                    </div>
                    <div class="relative flex w-[calc(100%-50px)] flex-col gap-1 md:gap-3 lg:w-[calc(100%-115px)]">
                        <div class="flex flex-grow flex-col gap-3">
                            <div class="min-h-10 flex flex-col items-start gap-4 whitespace-pre-wrap break-words">
                                <div class="markdown prose w-full break-words dark:prose-invert dark">
                                    {message_content}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub(crate) fn Messages(messages: ReadSignal<Vec<MessageModel>>) -> impl IntoView {
    view! {
        <For each=messages key=|m| (m.id, m.content.clone()) let:message>
            <Message message=message.to_owned().into()/>
        </For>
    }
}
