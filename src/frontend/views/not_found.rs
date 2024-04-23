use leptos::*;

#[component]
pub(crate) fn NotFound() -> impl IntoView {
    view! {
        <div class="overflow-hidden w-full h-screen relative flex dark:bg-gray-700 dark:text-gray-300">
            <h2>"Page Not Found"</h2>
        </div>
    }
}
