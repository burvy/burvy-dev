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

export function import_and_start(url, digest_url) {
    let started = loaded.get(url);
    if (!started) {
        started = (async () => {
            const mod = await import(url);
            await mod.default();
            // NEW
            let digest = "";
            if (digest_url) {
                const res = await fetch(digest_url + "?t=" + Date.now());
                const text = (await res.text()).trim();
                if (!/^[0-9a-f]{64}$/.test(text)) {
                    throw new Error(
                        `&{digest_url} did not return a digest` +
                        `(server running?): ${text.slice(0, 40)}`
                    );
                }
                digest = text;
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
    async fn import_and_start(url: &str, digest_url: &str) -> Result<JsValue, JsValue>;
}

/// Loads a lazy wasm experience, logging to the console if it fails rather
/// than leaving the user staring at a blank canvas with no explanation.
/// Requires a digest_url primarily for use in `game.rs`, just leave blank
/// if you do not need it
pub fn start_experience(url: &'static str, digest_url: &'static str) {
    leptos::task::spawn_local(async move {
        if let Err(e) = import_and_start(url, digest_url).await {
            web_sys::console::error_2(&format!("failed to load {url}:").into(), &e);
        }
    });
}
