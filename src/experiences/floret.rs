use leptos::prelude::*;

use crate::lazy;

/// The hangout. Reuses `#game-wrapper` so the canvas fills the page the same way
/// the other bevy experiences do.
#[component]
pub fn Floret() -> impl IntoView {
    let canvas = NodeRef::<leptos::html::Canvas>::new();

    // Wait for the canvas to exist before starting: bevy looks it up by id, and
    // starting first means it finds nothing and falls back to its own window.
    Effect::new(move |_| {
        if canvas.get().is_some() {
            lazy::start_experience("/floret/floret-wasm.js");
        }
    });

    view! {
        <div id="game-wrapper">
            <canvas node_ref=canvas id="floret-canvas">
                "Loading..."
            </canvas>
        </div>
    }
}
