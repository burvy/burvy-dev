use wasm_bindgen::prelude::*;

/// Entry point the site calls after dynamically importing this module.
/// `life_v2::run()` already guards against creating a second winit event
/// loop, so a repeat navigation to /life is harmless.
#[wasm_bindgen]
pub fn start() {
    life_v2::run();
}
