use leptos::prelude::*;

use crate::lazy;

#[component]
pub fn Venture() -> impl IntoView {
    let canvas = NodeRef::<leptos::html::Canvas>::new();

    Effect::new(move |_| {
        if canvas.get().is_some() {
            lazy::start_experience("/venture/venture-wasm.js");
        }
    });

    view! {
        <div id="game-wrapper">
            <canvas node_ref=canvas id="venture-canvas">
                "Loading..."
            </canvas>
        </div>
    }
}
