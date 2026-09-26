use leptos::prelude::*;

// Godot's web export bootstraps its own canvas + loading UI inside its
// generated html, unlike the wasm-bindgen modules (see lazy.rs) - so this
// just iframes that page directly instead of calling lazy::start_experience.
#[component]
pub fn Learn3D() -> impl IntoView {
    view! {
        <div id="game-wrapper">
            <iframe
                src="/3d-learn/3d-learn.html"
                style="width: 100%; height: 100%; border: none;"
            ></iframe>
        </div>
    }
}
