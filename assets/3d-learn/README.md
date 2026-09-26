# 3d-learn (Godot web export)

Not a Rust wasm module, so `build.ps1` doesn't build or clean this folder.

To update:
1. In Godot (project at `Z:\3. Godot Projects\3d-learn`): Project -> Export -> Web preset -> Export Project.
2. Export path: this folder, filename `3d-learn.html` (so output is `3d-learn.html`, `3d-learn.js`, `3d-learn.wasm`, `3d-learn.pck`, right here).
3. Delete this README's placeholder note once real files are here - `trunk` just needs *something* in this folder to copy, but stale docs are confusing.

`trunk` copies whatever's in here to `dist/3d-learn/` on the next site build (`.\build.ps1` or `trunk build`).
