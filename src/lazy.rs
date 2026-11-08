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
//
// While it loads, it keeps the page's loading screen (`overlay`, see
// components/wasm_experience.rs) up to date: the download's progress, then
// "starting", then fades it out once the game is running, or shows what went
// wrong. The wasm file is fetched here rather than by the module's own init, so
// the bytes can be counted as they arrive; it's handed over still streaming, so
// the browser compiles it while it downloads, same as before.
//
// A `versioned` module (sandbags, task 273) is loaded as `<module>.js?v=<tag>`, the tag
// read first from the `version.json` its deploy publishes beside it, past every cache.
// The URL then changes with every deploy, so no browser or CDN can hand a page an old
// cached module, even when the site bundle itself is the older one. No usable
// version.json (a local site): the plain URLs, as before.
#[wasm_bindgen(inline_js = r#"
const loaded = new Map();

const MB = 1024 * 1024;

// Updates the loading screen: its message, and the bar (a share from 0 to 1, or
// null when the size isn't known, which makes the bar sweep instead). Pages
// without a loading screen pass null for it
function show(overlay, detail, share) {
    if (!overlay) return;
    const bar = overlay.querySelector('.loading-bar');
    const fill = overlay.querySelector('.loading-bar-fill');
    overlay.querySelector('.loading-detail').textContent = detail;
    bar.classList.toggle('indeterminate', share === null);
    fill.style.width = share === null ? '' : `${Math.round(share * 100)}%`;
}

// The wasm file's response, counting its bytes into the loading screen as it
// streams in. Servers that compress it don't say how big it will be once
// unpacked, so without a usable size the bar just sweeps
async function fetchWithProgress(url, overlay) {
    const response = await fetch(url);
    if (!response.ok || !response.body) return response;
    const total = response.headers.get('Content-Encoding')
        ? 0
        : Number(response.headers.get('Content-Length')) || 0;
    let received = 0;
    const counted = response.body.pipeThrough(new TransformStream({
        transform(chunk, controller) {
            received += chunk.byteLength;
            const size = total ? ` of ${(total / MB).toFixed(1)}` : '';
            show(overlay, `Downloading ${(received / MB).toFixed(1)}${size} MB`,
                total ? Math.min(received / total, 1) : null);
            controller.enqueue(chunk);
        },
    }));
    return new Response(counted, {
        status: response.status,
        headers: { 'Content-Type': 'application/wasm' },
    });
}

// Fades the loading screen out, then takes it away entirely so it can't catch
// clicks meant for the game
function hide(overlay) {
    if (!overlay) return;
    overlay.classList.add('done');
    setTimeout(() => { overlay.style.display = 'none'; }, 400);
}

// The module's js and wasm URLs; for a versioned one, with `?v=<tag>` from its
// version.json: `module` (the wasm's own hash, so a client-only change busts too),
// else `build`
async function moduleUrls(url, versioned) {
    const wasm = url.replace(/\.js$/, '_bg.wasm');
    if (!versioned) return { js: url, wasm };
    try {
        const dir = url.slice(0, url.lastIndexOf('/') + 1);
        const response = await fetch(`${dir}version.json?t=${Date.now()}`, { cache: 'no-store' });
        if (response.ok) {
            const version = await response.json();
            const tag = version.module || version.build;
            if (typeof tag === 'string' && /^[0-9a-f]{8,64}$/i.test(tag)) {
                return { js: `${url}?v=${tag}`, wasm: `${wasm}?v=${tag}` };
            }
        }
    } catch (_) {
        // no version.json, or not JSON (a local site's fallback page): plain URLs
    }
    return { js: url, wasm };
}

export function import_and_start(url, overlay, versioned) {
    let started = loaded.get(url);
    if (!started) {
        started = (async () => {
            show(overlay, 'Downloading…', null);
            const urls = await moduleUrls(url, versioned);
            const mod = await import(urls.js);
            const wasm = new URL(urls.wasm, location.href);
            await mod.default({ module_or_path: fetchWithProgress(wasm, overlay) });
            show(overlay, 'Starting…', 1);
            mod.start();
        })();
        loaded.set(url, started);
    }
    return started.then(
        // two frames: the game draws its first one before the screen fades
        () => requestAnimationFrame(() => requestAnimationFrame(() => hide(overlay))),
        (error) => {
            if (overlay) overlay.classList.add('failed');
            show(overlay, `Couldn't load: ${error && error.message || error}`, 0);
            throw error;
        },
    );
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn import_and_start(
        url: &str,
        overlay: &JsValue,
        versioned: bool,
    ) -> Result<JsValue, JsValue>;
}

/// Loads a lazy wasm experience, logging to the console if it fails rather
/// than leaving the user staring at a blank canvas with no explanation.
pub fn start_experience(url: &'static str) {
    start(url, JsValue::NULL, false);
}

/// Loads a lazy wasm experience behind its loading screen (`overlay`, see
/// components/wasm_experience.rs), keeping it up to date until the game runs.
/// `versioned`: the module's deploy publishes a version.json beside it (see above)
pub fn start_with_loading_screen(url: &'static str, overlay: JsValue, versioned: bool) {
    start(url, overlay, versioned);
}

fn start(url: &'static str, overlay: JsValue, versioned: bool) {
    leptos::task::spawn_local(async move {
        if let Err(e) = import_and_start(url, &overlay, versioned).await {
            web_sys::console::error_2(&format!("failed to load {url}:").into(), &e);
        }
    });
}
