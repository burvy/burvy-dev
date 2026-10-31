use wasm_bindgen::prelude::*;

/// Entry point the site calls after dynamically importing this module.
/// `sandbags::run()` already guards against being started twice, so a repeat
/// navigation to /sandbags is harmless.
#[wasm_bindgen]
pub fn start() {
    sandbags::run();
}

