# burvy's portfolio site
A site to showcase a bunch of stuff in Rust

Also I get to test putting random stuff on websites!!!

# Building
To build quickly:
`.\build.ps1`  
To build a release build:
`.\build.ps1 -Deploy`

# Game
`cd crates/game-wasm`  
`trunk build --release`

`game-wasm` is a [cdylib](https://users.rust-lang.org/t/why-do-i-need-to-set-the-crate-type-to-cdylib-to-build-a-wasm-binary/93247) 
that wraps the `web-fps` crate so the site can `import()` it at runtime rather than 
linking the whole thing into the main bundle, which would cause extremely long initial 
loading times.  

`game-wasm` and `life-wasm`, and any other WASMs must be built before the site as `trunk` must 
copy the built executables into its `dist/` through the `game-wasm` `trunk` and then through the 
main `burvy-dev` `trunk`.

Note that `filehash = false` creates a stale cached `.js` file against the new `.wasm` which gives 
`LinkError`, but you can get around it by going to `DevTools` (CTRL + SHIFT + I) -> `Network` -> 
`Disable Cache`. The real fix should be implemented in the project itself though, to update the cache 
without needing to manually disable caching.  
`lazy.rs` imports `/game/game-wasm.js` by a fixed path, the filename, if hashed, would break on 
each rebuild.

Note that `data-wasm-opt="s"` takes minutes on a large WASM, so `trunk` writes the JS well before the WASM.  
There is a timestamp gap as a result of this.

# Cellular Automata
`cd crates/life-wasm`
`trunk build --release`

Systems:

Networking:
[WebTransport Server](docs/webtransport-server.md)
[WebTransport Client](docs/webtransport-client.md)

# Adding Items From Other Crates
To add a new big item, follow this checklist, and note, replace `<!name!> 
with your crate name (<!Name!> is your crate name but capitalized):

1. In the source crate, you must have a reusable entrypoint. For example, `run()` in your `lib.rs`. 
The `crate-type` must also be set to `rlib` so other crates can depend on it as a library.  

2. Create a new wrapper crate in `burvy-dev/crates/<!name!>-wasm`  
- `Cargo.toml`: `[lib] crate-type = ["cdylib"]`, dependencies: `<!name!> = { path = "../../../<!name!>" }` + `wasm-bindgen`.
Note that if you aren't me, the path is different. This would still be correct if you keep your project folders next to 
each other like I do.  
- `src/lib.rs`:
```rust
#[wasm_bindgen]
pub fn start() {
    <!name!>::run();
}
```  
- `Trunk.toml`: `dist = "../../assets/<!name!>"`, `filehash = false`, `no_sri = true`, `html_output = "_module.html"`.  

3. Add `"crates/<!name!>-wasm"` to `[workspace] members` in `Cargo.toml` in the root site  
Also add `<!name!>-wasm` to the `$Modules` array in `build.ps1` (and the dist-listing line if you want).  

4. Do these steps for the UI:
- New file `src/experiences/<!name!>.rs`: 
```rust
use leptos::prelude::*;

use crate::lazy;

#[component]
pub fn <!Name!>() -> impl IntoView {
    let canvas = NodeRef::<leptos::html::Canvas>::new();

    Effect::new(move |_| {
        if canvas.get().is_some() {
            lazy::start_experience("/<!name!>/<!name!>-wasm.js");
        }
    });

    view! {
        <div id="game-wrapper"> // game-wrapper fills the page
            <canvas node_ref=canvas id="<!name!>-canvas">
                "Loading..."
            </canvas>
        </div>
    }
}
```  

- Your crate must target the canvas we created, `id="<!name!>-canvas"`.  
For example, through `canvas-parent: Some("life-canvas".to_string())` like in `life-v2`.  
- Register the module: `pub mod <!name!>;` in `src/experiences/mod.rs`  
- Add an entry to the `EXPERIENCES` const array at the top of that `mod.rs`  
- You must draw an image, my convention is `2000x1000`. That is put in `assets/images`  
- Register a route in `src/app/app.rs`: `<Route path=path!("/<!name!>") 
view=experiences::<!name!>::<!Name!> />`

# Adding Normal Leptos Items Natively
The process is similar for adding normal leptos items.

1. Copy `test.rs` in `src/experiences` as a template for the leptos item, 
rename the file to `<!name!>` and the function inside to `<!Name!>`.

2. Add the item in `mod.rs` as an import and an `Experience` in `EXPERIENCES`.

3. Add the item as a new link in `src/app/app.rs`
