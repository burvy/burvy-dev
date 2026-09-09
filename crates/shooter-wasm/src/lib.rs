use wasm_bindgen::prelude::*;

/// Entry point the site calls after dynamically importing this module.
/// `shooter::run()` already guards against being started twice, so a repeat
/// navigation to /shooter is harmless.
#[wasm_bindgen]
pub fn start() {
    shooter::run();
}
