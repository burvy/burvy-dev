use wasm_bindgen::prelude::*;

/// Entry point the site calls after dynamically importing this module.
/// `floret::run()` already guards against a second start, so navigating back to
/// /floret is harmless.
#[wasm_bindgen]
pub fn start() {
    floret::run();
}
