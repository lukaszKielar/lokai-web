use leptos::*;

use crate::frontend::components::Conversation;

#[component]
pub(crate) fn Chat() -> impl IntoView {
    view! { <Conversation/> }
}
