use leptos::*;
use leptos_icons::Icon;
use leptos_router::A;

use crate::app::AppContext;
use crate::models;
use crate::server::api::get_conversations;

#[component]
fn Conversation(conversation: MaybeSignal<models::Conversation>) -> impl IntoView {
    let conversation = conversation.get();
    view! {
        <div class="flex flex-col gap-2 pb-2 text-gray-100 text-sm">
            <A
                href=format!("/c/{}", conversation.id)
                class="flex p-3 items-center gap-3 relative rounded-md hover:bg-[#2A2B32] cursor-pointer break-all hover:pr-4 group"
            >
                <Icon icon=icondata::LuMessageSquare class="h-4 w-4"/>
                <div class="flex-1 text-ellipsis align-middle h-6 overflow-hidden break-all relative">
                    {conversation.name}
                </div>
            </A>
        </div>
    }
}

#[component]
fn ConversationsLoading() -> impl IntoView {
    let div_cls = "h-2 w-2 bg-white rounded-full animate-bounce";
    view! {
        <div class="flex flex-col gap-2 pb-2 text-gray-100 text-sm">
            <div class="flex p-3 place-content-center gap-1 relative">
                <span class="sr-only">"Loading..."</span>
                <div class=format!("{div_cls} [animation-delay:-0.3s]")></div>
                <div class=format!("{div_cls} [animation-delay:-0.15s]")></div>
                <div class=format!("{div_cls}")></div>
            </div>
        </div>
    }
}

#[component]
fn Conversations() -> impl IntoView {
    // SAFETY: it's safe to unwrap because I provide context in App
    let AppContext {
        conversations,
        model: _,
    } = use_context().unwrap();

    let db_conversations = create_resource(|| (), |_| async { get_conversations().await.unwrap() });

    view! {
        <Transition fallback=move || {
            view! {
                <>
                    <ConversationsLoading/>
                </>
            }
        }>
            {if let Some(convs) = db_conversations.get() {
                conversations.set(convs);
            }}
            {move || {
                conversations
                    .get()
                    .into_iter()
                    .map(|c| {
                        view! { <Conversation conversation=c.into()/> }
                    })
                    .collect_view()
            }}

        </Transition>
    }
}

// TODO: I should probably accept Signal instead of loading all conversations every time
#[component]
pub(crate) fn Sidebar() -> impl IntoView {
    view! {
        <div class="scrollbar-trigger flex h-full w-full flex-1 items-start border-white/20">
            <nav class="flex h-full flex-1 flex-col space-y-1 p-2">
                <a
                    href="/"
                    class="flex py-3 px-3 items-center gap-3 rounded-md hover:bg-gray-500/10 transition-colors duration-200 text-white cursor-pointer text-sm mb-1 flex-shrink-0 border border-white/20"
                >
                    <Icon icon=icondata::LuMessageSquarePlus class="h-4 w-4"/>
                    "New chat"
                </a>
                <div class="flex-col flex-1 overflow-y-auto border-b border-white/20">
                    <Conversations/>
                </div>
                <a class="flex py-3 px-3 items-center gap-3 rounded-md hover:bg-gray-500/10 transition-colors duration-200 text-white cursor-pointer text-sm">
                    <Icon icon=icondata::LuTrash2 class="h-4 w-4"/>
                    "Clear conversations"
                </a>
                <a class="flex py-3 px-3 items-center gap-3 rounded-md hover:bg-gray-500/10 transition-colors duration-200 text-white cursor-pointer text-sm">
                    <Icon icon=icondata::LuSettings class="h-4 w-4"/>
                    "Settings"
                </a>
            </nav>
        </div>
    }
}
