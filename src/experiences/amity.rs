use leptos::prelude::*;

use crate::components::WasmExperience;

#[component]
pub fn Amity() -> impl IntoView {
    view! { <WasmExperience title="Amity" script="/amity/amity-wasm.js" canvas_id="amity-canvas" /> }
}
