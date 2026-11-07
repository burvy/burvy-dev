use leptos::prelude::*;

// single-file static html demo (no wasm bootstrap) - iframed directly,
// same pattern as learn3d.rs
#[component]
pub fn LpSphere() -> impl IntoView {
    view! {
        <div id="game-wrapper">
            <iframe
                src="/lp-sphere/index.html"
                style="width: 100%; height: 100%; border: none;"
            ></iframe>
        </div>
    }
}
