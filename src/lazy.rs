use wasm_bindgen::prelude::*;

// Dynamically imports a standalone wasm module and runs its `start` export.
//
// This is the whole point of the split: bevy and the game of life are no
// longer linked into the main bundle, so the browser only downloads them when
// someone actually opens /game or /life.
//
// wasm-bindgen can't emit a dynamic `import()` itself, so it goes through this
// snippet. The `loaded` map means a second visit to the same route reuses the
// already-instantiated module instead of re-instantiating the wasm.
#[wasm_bindgen(inline_js = r#"
const loaded = new Map();

export function import_and_start(url) {
    let started = loaded.get(url);
    if (!started) {
        started = (async () => {
            const mod = await import(url);
            await mod.default();
            // NEW
            let digest = "";
            if (digest_url) {
                const res = await fetch(digest_url + "?t=" + Date.now())
                digest = (await res.text()).trim();
            }
            mod.start(digest);
        })();
        loaded.set(url, started);
    }
    return started;
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn import_and_start(url: &str) -> Result<JsValue, JsValue>;
}

/// Loads a lazy wasm experience, logging to the console if it fails rather
/// than leaving the user staring at a blank canvas with no explanation.
pub fn start_experience(url: &'static str) {
    leptos::task::spawn_local(async move {
        if let Err(e) = import_and_start(url).await {
            web_sys::console::error_2(&format!("failed to load {url}:").into(), &e);
        }
    });
}
