use leptos::*;

#[component]
pub(crate) fn Home() -> impl IntoView {
    view! {
        <div class="flex max-w-full dark:bg-gray-800">
            <div class="h-10">A</div>
            <div class="h-10">B</div>
            <div class="h-10">C</div>
        </div>
    }
}
