use leptos::prelude::*;

use crate::components::WasmExperience;

/// The hangout
#[component]
pub fn Floret() -> impl IntoView {
    view! { <WasmExperience title="Floret" script="/floret/floret-wasm.js" canvas_id="floret-canvas" /> }
}
