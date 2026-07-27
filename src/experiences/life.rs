use leptos::prelude::*;

use crate::lazy;

#[component]
pub fn Life() -> impl IntoView {
    Effect::new(|_| {
        // winit mounts into #life-canvas, so let the view paint first
        request_animation_frame(|| {
            lazy::start_experience("/life/life-wasm.js");
        });
    });

    view! {
        <div id="life-wrapper">
            <div id="life-canvas"></div>
        </div>
        <p>"space - pause, left click - birth, right click - kill, scroll - change speed"</p>
    }
}
