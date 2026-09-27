use leptos::prelude::*;

use crate::lazy;

#[component]
pub fn Amity() -> impl IntoView {
    let canvas = NodeRef::<leptos::html::Canvas>::new();

    Effect::new(move |_| {
        if canvas.get().is_some() {
            lazy::start_experience("/amity/amity-wasm.js");
        }
    });

    view! {
        <div id="game-wrapper">
            <canvas node_ref=canvas id="amity-canvas">
                "Loading..."
            </canvas>
        </div>
    }
}
