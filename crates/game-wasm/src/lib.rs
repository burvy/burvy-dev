use wasm_bindgen::prelude::*;

/// Entry point the site calls after dynamically importing this module.
/// `game::run()` already guards against being started twice, so a repeat
/// navigation to /game is harmless.
#[wasm_bindgen]
pub fn start() {
    // TODO: add an actual digest here
    game::run(String::new());
}
