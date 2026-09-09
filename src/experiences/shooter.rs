use leptos::prelude::*;

use crate::lazy;

#[component]
pub fn Shooter() -> impl IntoView {
    let canvas = NodeRef::<leptos::html::Canvas>::new();

    Effect::new(move |_| {
        if canvas.get().is_some() {
            lazy::start_experience("/shooter/shooter-wasm.js");
        }
    });

    view! {
        <div id="game-wrapper">
            <canvas node_ref=canvas id="shooter-canvas">
                "Loading..."
            </canvas>
        </div>
    }
}
