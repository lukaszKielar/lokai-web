use leptos::*;
use leptos_icons::Icon;

#[component]
pub(crate) fn Sidebar() -> impl IntoView {
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
