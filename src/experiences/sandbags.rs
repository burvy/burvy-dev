use leptos::prelude::*;

use crate::components::WasmExperience;

// TODO(burvy): nothing serves /sandbags/sandbags-wasm.js yet, so this page
// doesn't load the game until the web build is wired up (see the task 018 report).
// versioned: `build.ps1 -Deploy` writes /sandbags/version.json, and the module is loaded
// by it, so a stale cached game can't outlive a server update (sandbags task 273).
#[component]
pub fn Sandbags() -> impl IntoView {
    view! { <WasmExperience title="Sandbags" script="/sandbags/sandbags-wasm.js" canvas_id="sandbags-canvas" versioned=true /> }
}
