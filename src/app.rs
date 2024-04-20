use std::str::FromStr;

use crate::models::{Message, Role};
use crate::server::api::{get_conversation_messages, AskAssistant};

use icondata::Icon;
use leptos::ev::SubmitEvent;
use leptos::*;
use leptos_icons::Icon;
use leptos_meta::*;
use uuid::Uuid;

#[component]
fn Sidebar() -> impl IntoView {
    view! {
        <div class="scrollbar-trigger flex h-full w-full flex-1 items-start border-white/20">
            <nav class="flex h-full flex-1 flex-col space-y-1 p-2">
                <a class="flex py-3 px-3 items-center gap-3 rounded-md hover:bg-gray-500/10 transition-colors duration-200 text-white cursor-pointer text-sm mb-1 flex-shrink-0 border border-white/20">
                    <Icon icon=icondata::LuMessageSquarePlus class="h-4 w-4"/>
                    New chat
                </a>
                <div class="flex-col flex-1 overflow-y-auto border-b border-white/20">
                    <div class="flex flex-col gap-2 pb-2 text-gray-100 text-sm">
                        <a class="flex py-3 px-3 items-center gap-3 relative rounded-md hover:bg-[#2A2B32] cursor-pointer break-all hover:pr-4 group">
                            <Icon icon=icondata::LuMessageSquare class="h-4 w-4"/>
                            <div class="flex-1 text-ellipsis max-h-5 overflow-hidden break-all relative">
                                Conversation
                                <div class="absolute inset-y-0 right-0 w-8 z-10 bg-gradient-to-l from-gray-900 group-hover:from-[#2A2B32]"></div>
                            </div>
                        </a>
                    </div>
                </div>
                <a class="flex py-3 px-3 items-center gap-3 rounded-md hover:bg-gray-500/10 transition-colors duration-200 text-white cursor-pointer text-sm">
                    <Icon icon=icondata::LuTrash2 class="h-4 w-4"/>
                    Clear conversations
                </a>
                <a class="flex py-3 px-3 items-center gap-3 rounded-md hover:bg-gray-500/10 transition-colors duration-200 text-white cursor-pointer text-sm">
                    <Icon icon=icondata::LuSettings class="h-4 w-4"/>
                    Settings
                </a>
            </nav>
        </div>
    }
}

#[component]
fn MessageComponent(message: ReadSignal<Message>) -> impl IntoView {
    let is_user = move || match Role::from(message.get_untracked().role) {
        Role::Assistant | Role::System => false,
        Role::User => true,
    };

    let message_class = move || {
        let message_role_class = match is_user() {
            true => "dark:bg-gray-800",
            false => "bg-gray-50 dark:bg-[#444654]",
        };
        format!("group w-full text-gray-800 dark:text-gray-100 border-b border-black/10 dark:border-gray-900/50 {message_role_class}")
    };

    let message_icon = move || {
        let class = "h-4 w-4 text-white";
        match is_user() {
            true => view! { <Icon icon=icondata::LuUser class=class/> },
            false => view! { <Icon icon=icondata::LuBot class=class/> },
        }
    };

    let message_content_class = move || {
        let message_content = message.get_untracked().content;
        if !is_user() && message_content == "" {
            view! { <Icon icon=icondata::LuTextCursorInput class="h-6 w-6 animate-pulse"/> }
                .into_view()
        } else {
            view! { <p>{message_content}</p> }.into_view()
        }
    };

    view! {
        <div class=message_class()>
            <div class="text-base gap-4 md:gap-6 md:max-w-2xl lg:max-w-xl xl:max-w-3xl flex lg:px-0 m-auto w-full">
                <div class="flex flex-row gap-4 md:gap-6 md:max-w-2xl lg:max-w-xl xl:max-w-3xl p-4 md:py-6 lg:px-0 m-auto w-full">
                    <div class="w-8 flex flex-col relative items-end">
                        <div class="relative h-7 w-7 p-1 rounded-full text-white flex items-center justify-center bg-black/75 text-opacity-100r">
                            {message_icon()}
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
                                    {message_content_class()}
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
fn Conversation() -> impl IntoView {
    let (conversation_id, _) =
        create_signal(Uuid::from_str("1ec2aa50-b36d-4bf6-a9d8-ef5da43425bb").unwrap());

    let (model, set_model) = create_signal(String::from("mistral:7b"));

    let db_messages = create_resource(
        || (),
        |_| async {
            get_conversation_messages(
                Uuid::from_str("1ec2aa50-b36d-4bf6-a9d8-ef5da43425bb").unwrap(),
            )
            .await
            .unwrap()
        },
    );

    let (messages, set_messages) = create_signal(Vec::<Message>::new());
    let (user_prompt, set_user_prompt) = create_signal(String::new());

    let send_user_prompt = create_server_action::<AskAssistant>();
    let assistant_response_value = send_user_prompt.value();

    let (button_disabled, set_button_disabled) = create_signal(true);

    create_effect(move |_| {
        if user_prompt().len() == 0 || send_user_prompt.pending().get() {
            set_button_disabled(true)
        } else {
            set_button_disabled(false)
        };
    });

    create_effect(move |_| {
        if let Some(response) = assistant_response_value.get() {
            let assistant_response = response.unwrap();
            set_messages.update(|msgs| msgs.push(assistant_response));
        }
    });

    view! {
        <div class="flex max-w-full flex-1 flex-col">
            <div class="relative h-full w-full transition-width flex flex-col overflow-hidden items-stretch flex-1">
                <div class="flex-1 overflow-hidden">
                    <div class="react-scroll-to-bottom--css-ikyem-79elbk h-full dark:bg-gray-800">
                        <div class="react-scroll-to-bottom--css-ikyem-1n7m0yu">
                            <div class="flex flex-col items-center text-sm bg-gray-800">
                                <Transition fallback=move || {
                                    view! {
                                        <div class="flex w-full items-center justify-center gap-1 border-b border-black/10 bg-gray-50 p-3 text-gray-500 dark:border-gray-900/50 dark:bg-gray-700 dark:text-gray-300">
                                            "Loading initial data..."
                                        </div>
                                    }
                                }>
                                    {if let Some(messages) = db_messages.get() {
                                        set_messages(messages);
                                    }}
                                    <div class="flex w-full items-center justify-center gap-1 border-b border-black/10 bg-gray-50 p-3 text-gray-500 dark:border-gray-900/50 dark:bg-gray-700 dark:text-gray-300">
                                        "Model: " <b>{model}</b>
                                    </div>
                                    {move || {
                                        messages()
                                            .iter()
                                            .map(|m| {
                                                let (message, _) = create_signal(m.clone());
                                                view! { <MessageComponent message/> }
                                            })
                                            .collect_view()
                                    }} <div class="w-full h-32 md:h-48 flex-shrink-0"></div>
                                // <div ref=bottomOfChatRef></div>
                                </Transition>
                            </div>
                            <div class="flex flex-col items-center text-sm dark:bg-gray-800"></div>
                        </div>
                    </div>
                </div>
                <div class="absolute bottom-0 left-0 w-full border-t md:border-t-0 dark:border-white/20 md:border-transparent md:dark:border-transparent md:bg-vert-light-gradient bg-white dark:bg-gray-800 md:!bg-transparent dark:md:bg-vert-dark-gradient pt-2">
                    <form class="stretch mx-2 flex flex-row gap-3 last:mb-2 md:mx-4 md:last:mb-6 lg:mx-auto lg:max-w-2xl xl:max-w-3xl">
                        <div class="relative flex flex-col h-full flex-1 items-stretch md:flex-col">
                            // {errorMessage ? (
                            // <div class="mb-2 md:mb-0">
                            // <div class="h-full flex ml-1 md:w-full md:m-auto md:mb-2 gap-0 md:gap-2 justify-center">
                            // <span class="text-red-500 text-sm">{errorMessage}</span>
                            // </div>
                            // </div>
                            // ) : null}
                            <div class="flex flex-col w-full py-2 flex-grow md:py-3 md:pl-4 relative border border-black/10 bg-white dark:border-gray-900/50 dark:text-white dark:bg-gray-700 rounded-md shadow-[0_0_10px_rgba(0,0,0,0.10)] dark:shadow-[0_0_15px_rgba(0,0,0,0.10)]">
                                <textarea
                                    on:input=move |ev| {
                                        set_user_prompt(event_target_value(&ev));
                                    }

                                    type="text"
                                    placeholder="Message assistant..."
                                    prop:value=user_prompt
                                    // ref={textAreaRef} <- important for auto scrolling
                                    // tabIndex={0} <- no idea
                                    // data-id="root" <- no idea
                                    // style={{ <- no idea
                                    // height: "24px", <- no idea
                                    // maxHeight: "200px", <- no idea
                                    // overflowY: "hidden", <- no idea
                                    // }} <- no idea
                                    rows=1
                                    placeholder="Send a message..."
                                    class="m-0 w-full resize-none border-0 bg-transparent p-0 pr-7 focus:ring-0 focus-visible:ring-0 dark:bg-transparent pl-2 md:pl-0 h-[24px] max-h-[200px] overflow-y-hidden"
                                >// onKeyDown={handleKeypress}
                                </textarea>
                                // disabled={isLoading || message?.length === 0}
                                // onClick={sendMessage}
                                <button
                                    class="absolute p-1 rounded-md bottom-1.5 md:bottom-2.5 bg-transparent disabled:bg-gray-500 right-1 md:right-2 disabled:opacity-40"
                                    on:click=move |ev| {
                                        ev.prevent_default();
                                        let user_message = Message::user(
                                            user_prompt(),
                                            conversation_id(),
                                        );
                                        let user_message_clone = user_message.clone();
                                        if user_message.content != "" {
                                            set_messages.update(|msgs| msgs.push(user_message_clone));
                                            send_user_prompt.dispatch(AskAssistant { user_message });
                                            set_user_prompt("".to_string());
                                        }
                                    }

                                    disabled=button_disabled
                                >

                                    <Icon icon=icondata::LuSend class="h-4 w-4 mr-1 text-white "/>
                                </button>
                            </div>
                        </div>
                    </form>
                    <div class="px-3 pt-2 pb-3 text-center text-xs text-black/50 dark:text-white/50 md:px-4 md:pt-3 md:pb-6">
                        <span>Enjoy your self-hosted LokAI!</span>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // TODO: list of conversations should be loaded from the server in reaction to changes
    // TODO: separately read conversation and messages
    let conversation_id = Uuid::from_str("1ec2aa50-b36d-4bf6-a9d8-ef5da43425bb").unwrap();
    let db_messages = create_resource(
        || (),
        |_| async {
            get_conversation_messages(
                Uuid::from_str("1ec2aa50-b36d-4bf6-a9d8-ef5da43425bb").unwrap(),
            )
            .await
            .unwrap()
        },
    );
    let (messages, set_messages) = create_signal(Vec::<Message>::new());

    let send_user_prompt = create_server_action::<AskAssistant>();
    let assistant_response_value = send_user_prompt.value();

    create_effect(move |_| {
        if let Some(response) = assistant_response_value.get() {
            let assistant_response = response.unwrap();
            set_messages.update(|msgs| msgs.push(assistant_response));
        }
    });

    let messages_view = move || {
        messages()
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
    };

    let (user_prompt, set_user_prompt) = create_signal(String::new());

    view! {
        <Title text="Welcome to LokAI!"/>
        <Stylesheet id="leptos" href="/pkg/lokai.css"/>
        <Meta charset="UTF-8"/>

        <main class="overflow-hidden w-full h-screen relative flex">
            <div class="dark hidden flex-shrink-0 bg-gray-900 md:flex md:w-[260px] md:flex-col">
                <div class="flex h-full min-h-0 flex-col ">
                    <Sidebar/>
                </div>
            </div>
            <Conversation/>

        // <div class="flex flex-col h-screen bg-gray-50 place-items-center">
        // <div class="flex-1 w-[720px] bg-gray-100">
        // <Transition fallback=move || view! { <p>"Loading initial data..."</p> }>
        // <div class="flex flex-col space-y-2 p-2">

        // {
        // if let Some(messages) = db_messages.get() {
        // set_messages(messages)
        // }
        // messages_view
        // }

        // </div>
        // </Transition>
        // </div>
        // <div class="flex-none h-20 w-[720px] bottom-0 place-items-center justify-center items-center bg-gray-200">
        // <form on:submit=move |ev: SubmitEvent| {
        // ev.prevent_default();
        // let user_message = Message::user(user_prompt(), conversation_id);
        // let user_message_clone = user_message.clone();
        // if user_message.content != "" {
        // set_messages.update(|msgs| msgs.push(user_message_clone));
        // send_user_prompt.dispatch(AskAssistant { user_message });
        // set_user_prompt("".to_string());
        // }
        // }>

        // <div class="flex items-center p-2">
        // <input
        // on:input=move |ev| {
        // set_user_prompt(event_target_value(&ev));
        // }

        // class="border border-gray-300 rounded-lg px-4 py-2 w-full"
        // type="text"
        // placeholder="Message assistant..."
        // prop:value=user_prompt
        // />
        // <button
        // class="ml-2 bg-blue-500 hover:bg-blue-700 text-white px-4 py-2 rounded-lg"
        // type="submit"
        // >
        // Send
        // </button>
        // </div>
        // </form>
        // </div>
        // </div>
        </main>
    }
}
