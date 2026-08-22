# burvy's portfolio site
A site to showcase a bunch of stuff in Rust

Also I get to test putting random stuff on websites!!!

# Building
Build with:  
`./build.ps1`

# Game
`cd crates/game-wasm`  
`trunk build --release`

`game-wasm` is a [cdylib](https://users.rust-lang.org/t/why-do-i-need-to-set-the-crate-type-to-cdylib-to-build-a-wasm-binary/93247) 
that wraps the `web-fps` crate so the site can `import()` it at runtime rather than 
linking the whole thing into the main bundle, which would cause extremely long initial 
loading times.  

`game-wasm` and `life-wasm`, and any other WASMs must be built before the site as `trunk` must 
copy the built executables into its `dist/`. 

Note that `filehash = false` creates a stale cached `.js` file against the new `.wasm` which gives 
`LinkError`, but you can get around it by going to `DevTools` (CTRL + SHIFT + I) -> `Network` -> 
`Disable Cache`. The real fix should be implemented in the project itself though, to update the cache 
without needing to manually disable caching. 

Note that `data-wasm-opt="s"` takes minutes on a large WASM, so `trunk` writes the JS well before the WASM.  
There is a timestamp gap as a result of this.

# Cellular Automata
`cd crates/life-wasm`
`trunk build --release`

Systems:

Networking:
[WebTransport Server](docs/webtransport-server.md)
[WebTransport Client](docs/webtransport-client.md)
