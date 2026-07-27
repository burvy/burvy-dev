use leptos::prelude::*;

use crate::lazy;

#[component]
pub fn Game() -> impl IntoView {
    let canvas = NodeRef::<leptos::html::Canvas>::new();

    Effect::new(move |_| {
        if canvas.get().is_some() {
            lazy::start_experience("/game/game-wasm.js");
        }
    });

    view! {
        <div id="game-wrapper">
            <canvas node_ref=canvas id="game-canvas">
                "Loading..."
            </canvas>
        </div>
    }
}
