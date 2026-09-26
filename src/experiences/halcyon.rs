use leptos::prelude::*;

#[component]
pub fn Halcyon() -> impl IntoView {
    view! {
        <div id="game-wrapper">
            <iframe
                src="/halcyon/halcyon.html"
                style="width: 100%; height: 100%; border: none;"
            ></iframe>
        </div>
    }
}
