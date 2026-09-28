use leptos::prelude::*;

use crate::components::WasmExperience;

#[component]
pub fn Game() -> impl IntoView {
    view! { <WasmExperience title="Game" script="/game/game-wasm.js" canvas_id="game-canvas" /> }
}
