use leptos::prelude::*;

#[component]
pub fn AdventurePage() -> impl IntoView {
    view! {
        <div class="adventure-page">
            <iframe
                class="adventure-frame"
                src="/adventure-dist/index.html"
                title="Adventure"
            >
                "Adventure failed to load."
            </iframe>
        </div>
    }
}
