use leptos::prelude::*;

use crate::components::WasmExperience;

// TODO(burvy): nothing serves /sandbags/sandbags-wasm.js yet, so this page
// doesn't load the game until the web build is wired up (see the task 018 report).
#[component]
pub fn Sandbags() -> impl IntoView {
    view! { <WasmExperience title="Sandbags" script="/sandbags/sandbags-wasm.js" canvas_id="sandbags-canvas" /> }
}
