use leptos::prelude::*;

use crate::lazy;

#[component]
pub fn Life() -> impl IntoView {
    Effect::new(|_| {
        // winit mounts into #game-wrapper, so let the view paint first
        request_animation_frame(|| {
            lazy::start_experience("/life/life-wasm.js");
        });
    });

    view! {
        <div id="game-wrapper">
            <p class="overlay-controls">"space - pause, left click - birth, right click - kill, scroll - change speed"</p>
        </div>
    }
}
