# burvy's portfolio site
A site to showcase a bunch of stuff in Rust

Also I get to test putting random stuff on websites!!!

# Building
Run the menu:
`.\menu.ps1`  
It lists everything buildable (wasm modules, burvy.dev, statesmc.us, adventure, servers)
and you pick what to rebuild this time, e.g. `1 7` or `10-15`. Output goes to `deploy\`,
and it lists which files differ from the live sites.
To build quickly:  
`.\build.ps1`  
To build a release build (site + burvy-dev's own `server.exe`, gathered into `deploy\`):  
`.\build.ps1 -Deploy`  
To test locally against `http://localhost:8080` (also gathers every linked project's dev-build
server executable into `dev-servers\`, so you can run whichever one you're testing):  
`.\build.ps1 -Dev`  

To only build one wasm:  
`.\build.ps1 <!name!>-wasm`  

If you're only working on the site, honestly just use:  
`trunk serve`  
This just won't build the linked crates at all.

If you are importing Godot games, just do:  
`trunk build --release`

Per-project multiplayer servers (`amity-server`, `floret-server`, ...) still ship for real
through each project's own `go.ps1 -release`, which calls this script for just its wasm module.
`-Deploy`/`-Dev` here only cover what burvy-dev itself owns (the site, and its own server).

# StatesMC website (statesmc.us)
`crates/statesmc` is the website for the Minecraft server, moved here from `site-website`.
It is a **separate site on its own domain**, not part of burvy.dev: it builds on its own into
`deploy\statesmc\dist\` (menu option, or `.\build.ps1 -Deploy`), and that folder is what
statesmc.us serves.
- Preview: `cd crates/statesmc` then `trunk serve` (http://localhost:8090).
- It builds with its own `CARGO_TARGET_DIR` (`target\statesmc`): trunk shares
`target\wasm-bindgen\` across the workspace and would otherwise add burvy-dev's js snippets
to statesmc's page.
- `adventure-dist/` is the adventure game (`..\adventure`), built by the menu's adventure
option and committed here.

# Programming Languages Group at Penn State
[PL Group](https://sites.psu.edu/plgroup)
## Accounts (sign-in, settings, mailing list)
### How sign-in works
1. Google's button gives the page a signed *credential*.
2. The page POSTs it to `/auth/google`.
3. The server checks it with Google's `tokeninfo` endpoint. 
It must be meant for our `CLIENT_ID` and have a verified 
email.  
`hd == "psu.edu"` gets the PSU badge, not necessary but neat.
4. The server creates or updates the user and returns a random session 
token. Sessions last 90 days (can be configured)
5. The page saves the token in localStorage (`plgroup-token`) 
and sends it as `Authorization: Bearer <token>` on later 
requests. It uses a token rather than a cookie because cookies 
from another site get blocked.
6. A handler that takes `User` requires a valid token, 
and one that takes `Admin` requires an admin (`OWNER` or anyone 
in the `admins` table). Admin status is checked on every request.  
De-admining someone means their next admin-needed request gets rejected 
instead of next login

### Running locally
- Server: `cd plgroup-server && cargo run --features dev-local` 
(http, `plgroup-dev.db`, allows localhost:8702/8080)
- Page: `cd crates/plgroup && trunk serve --port 8702`. 
8702 is the origin Google allows. On `localhost` the page 
calls `http://localhost:3120` automatically.

### Adding things
#### new endpoint
1. Put the new request or reply struct in `crates/plgroup-api`.
2. Write the handler in the server module it belongs to. 
Take `User` or `Admin` as an argument to control access if desired.
3. Add a `.route(...)` in `main.rs`.
4. On the page: 
`json::<T>(authed(Request::get(&api("/path"))).build().unwrap()).await`.

#### new user setting
1. Add a field to `Settings` in `plgroup-api`.
2. In `db.rs`, add the column to `SCHEMA`. 
**The live DB already exists, so also run `ALTER TABLE users ADD COLUMN ...
DEFAULT 0` on it.** `CREATE TABLE IF NOT EXISTS` won't add it for existing users!
3. In `auth.rs`, add it to `user_columns!`, `User`, and `User::me()`.
4. Write it in `put_settings`'s `UPDATE`.
5. Add a control in `account/settings.rs`.

#### A new page route
- Add the route in `lib.rs`, plus a matching WordPress page on sites.psu.edu, 
or it 404s there.
- Links between sibling routes: use `<A href="../foo">`, 
not `"foo"`. Relative links resolve against the current route.

### NOTES
#### New site origin?
Update `ORIGINS` in `main.rs` and Google Cloud's Authorized JavaScript Origins (project
plgroup-510600).
#### sqlx 0.9 only accepts static SQL strings.
Build shared fragments with a macro and `concat!` (see 
`user_columns!`), never `format!`.
#### `OWNER`** in `admin.rs` is always an admin and can't be removed. 
Other admins are added from the settings page.
#### Errors: the server returns `(StatusCode, "short message")`, 
and the page shows that message as-is. Real errors 
go through `internal()`, which logs the details and 
sends only "server error".
#### Data:`C:\burvy\data\plgroup\plgroup.db` on the server PC. 
Back up that file.
#### Cloudflare: the CNAME `plgroup-api` is proxied
Proxied with the Origin Rule "port 3120" and the Configuration Rule 
"Full (strict)" for this host only (the zone default stays Flexible). 
TLS uses a Cloudflare Origin Certificate at `C:\burvy\certs\plgroup-api.burvy.dev\`.
#### Google app is published. 
Adding a logo in Branding would force Google verification.

### Known issues
- `Auth::new` (`account/mod.rs`) clears the token on any `/me` error, 
so a server restart signs people out. It 
should only clear on a 401, which needs `send` to return the status.
- CORS and localStorage are scoped to all of 
`https://sites.psu.edu`, so other PSU WordPress sites share that origin.
Fix this before storing anything more sensitive (submissions, posts).


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

# Primer
[Primer](../primer) is a picture-book maths app (its own Rust project, next to this one).
It shows up here as the `/primer` experience.

## How it is wired
- `primer-web` (`../primer/crates/web`) is a library: `primer_web::mount(element, Config)`
puts the app inside an element. `Config` is where the server (`api`) and the drawings/sounds
(`assets`) are.
- `crates/primer-wasm` is the wrapper (see "Adding Items From Other Crates"). Its `start()`
mounts Primer with `api: ""` (the site's own address) and `assets: "/primer/assets"`.
Its `index.html` also copies Primer's css, js and `assets/` next to the wasm, so they end
up in `/primer/`.
- `src/experiences/primer.rs` is the page. Primer's css is written for a page of its own
(it styles `body`), so the page adds it when opened and takes it off when left.
Primer is built once and kept, so going away and back does not sign you out.
- Primer has no URL router (one signal picks the page), so it cannot clash with this site's.
- The card image `assets/images/primer.png` is a **placeholder** (2000x1000), to be drawn.

## Running locally
Primer's server needs to be running too, on port 3140 (`PRIMER_DEV_SIGNIN=1` is the local
test sign-in, **never** use it on a real server):
```
cd ..\primer
$env:PRIMER_ADDR = "127.0.0.1:3140"
$env:DATABASE_URL = "sqlite://primer-dev.db?mode=rwc"
$env:PRIMER_DEV_SIGNIN = "1"
cargo run -p primer-server
```
Then, in this folder, build and serve the site (`-Dev` also builds Primer's server into
`dev-servers\primer-server.exe`; that one needs the same variables, plus
`PRIMER_CONTENT` and `PRIMER_ASSETS` pointing at `..\primer\content` and `..\primer\assets`):
```
.\build.ps1 primer-wasm
trunk serve --release
```
Open http://localhost:8080/primer. `Trunk.toml` sends `/api/` to port 3140, so the page and
the server look like one site. Only primer's wasm: `.\build.ps1 primer-wasm`.

## Known issues
- Going live needs `/api/` on the site's address to reach Primer's server (a reverse proxy,
or CORS on Primer's server plus an `api` address in `crates/primer-wasm/src/lib.rs`).
Primer's sign-in on the real site is Google's (`PRIMER_GOOGLE_CLIENT_ID`), with this
site's origin allowed in Google Cloud.
- Primer's css is not scoped to its own element, only to the time the page is open.

## Going live
The server runs on the server PC like `plgroup-server` (own port, https from a Cloudflare
origin certificate, CORS for this site). Its settings are environment variables
(details: Primer's `docs/DEPLOY.md`, part J). Checklist:
1. Cloudflare: a **proxied CNAME** `primer-api` (to the same target as `plgroup-api`).
2. Cloudflare **Origin Rule** for `primer-api.burvy.dev`: destination port **3140**.
3. Cloudflare **Configuration Rule** "Full (strict)" for this host only (zone default stays Flexible).
4. Origin certificate for `primer-api.burvy.dev` (Cloudflare, Origin Server) saved as
`C:\burvy\certs\primer-api.burvy.dev\cert.pem` and `key.pem`.
5. Data: `C:\burvy\data\primer\` (the database, back it up). Jev key (raw key only):
`C:\burvy\secrets\primer\jev-key.env`.
6. Google Cloud: in the OAuth client, add `https://burvy.dev` to **Authorized JavaScript
origins** (the client id is what `PRIMER_GOOGLE_CLIENT_ID` is set to).
7. Router and Windows firewall: open TCP **3140** to the server PC.
8. Run `primer-server.exe` (`.\build.ps1 -Deploy` puts it in `deploy\`) from a folder that has
Primer's `content\` and `assets\`, with these set:
```
PRIMER_ADDR=0.0.0.0:3140
PRIMER_TLS_CERT=C:\burvy\certs\primer-api.burvy.dev\cert.pem
PRIMER_TLS_KEY=C:\burvy\certs\primer-api.burvy.dev\key.pem
PRIMER_ALLOWED_ORIGINS=https://burvy.dev
PRIMER_GOOGLE_CLIENT_ID=<the Google client id>
DATABASE_URL=sqlite://C:/burvy/data/primer/primer.db?mode=rwc
PRIMER_JEV_KEY_FILE=C:\burvy\secrets\primer\jev-key.env
PRIMER_CONTENT=content
PRIMER_ASSETS=assets
```
9. **`PRIMER_DEV_SIGNIN` must NOT be set** on the live server (with a Google client id the
server refuses to start if it is).
10. The server reads the certificate once at start, so the scheduled restart (which refreshes
certs) is enough. New site origin? Update `PRIMER_ALLOWED_ORIGINS` and Google's origins.
11. The site picks the server by hostname (`crates/primer-wasm/src/lib.rs`): `localhost` uses
the Trunk proxy, anywhere else `https://primer-api.burvy.dev`.

# Golden
[Golden](https://github.com/burvy/golden) finds where and when to film the golden hour
(sun, horizon and rooftops on a map). It is its own Rust project (from HackPrinceton
Fall 2026), kept at `Z:\3Projects\golden`, so **not** next to this folder like the others.
It shows up here as the `/golden` experience.

## How it is wired
- Golden's web front end (golden's `web/`) is a static page. `index.html` is the current
page (code in `v2/js/`), `classic/` is golden's first page (it uses `js/` and `golden-hero.js`,
and the current page links to it), and `v2/` only redirects to `index.html`. It is copied as-is into `assets/golden-app/` and that copy is committed, the same as
`metric-balls` and `3d-learn`. `index.html` has a `copy-dir` for it, so it ends up in
`/golden-app/`. (Not `/golden/`: if a folder has the same name as the route, `trunk serve`
redirects `/golden` to that folder, so a reload would open golden's page on its own, without
the site. `/golden/api/` stays only the proxy.)
- To refresh the copy after golden's web changes, take it from golden's `main` on GitHub,
not from a local checkout (that may be behind or have unfinished work):
```
git -C Z:\3Projects\golden fetch origin
Remove-Item assets\golden-app\* -Recurse -Force
git -C Z:\3Projects\golden archive origin/main web | tar -x -C assets\golden-app --strip-components=1 --exclude .DS_Store
```
- `src/experiences/golden.rs` is the page. Golden's page styles `body` and loads three.js
itself, so it is iframed (like `metric_balls.rs`), which keeps its css off this site.
- The iframe opens `/golden-app/index.html?api=/golden`. Golden's page reads `?api=` as its
API address (without it, it uses its own default, `http://localhost:8787`), so here it calls
`/golden/api/...` on this site's address. Primer already has `/api/`, so golden gets its
own prefix.
- `Trunk.toml` sends `/golden/api/` to golden's server at `127.0.0.1:8787/api/`.
- If golden's server does not answer, the page shows saved example data and says so in its
header ("Example data (server not reachable)"), so the page never breaks. (The classic page
does the same per endpoint: "Sample data for ...".)
- The card image `assets/images/golden.png` is a **placeholder** (2000x1000), to be drawn.

## Running locally
Golden's server needs to be running too, on port 8787 (by default it only listens on
`127.0.0.1`). It reads its keys from environment variables, or from key files:
`VEXCEL_ENV_FILE` and `ANTHROPIC_ENV_FILE` can each point at a file holding just the key on
one line (as Vexcel hands it out) or a `.env` with `NAME=...` lines, and other keys come from
golden's shared `env\.env` (see golden's README, "Configuration"). Without a Vexcel key, areas
already in its cache still work; without an Anthropic key, golden plans the shot offline:
```
cd Z:\3Projects\golden
.\run.ps1
```
Then, in this folder:
```
trunk serve --release
```
Open http://localhost:8080/golden. If golden's header says "Example data", the page could not
reach golden's server.

## Known issues
- The copy in `assets/golden-app/` does not update by itself; refresh it (above) when golden's
web changes.
- Golden's page loads three.js, fonts and map tiles from other sites (CDNs, Esri), so it needs
the internet even locally.

## Going live
Not done yet. burvy.dev is static, so on the live site `/golden/api/` (on the site's own
address) has to reach golden's server.

**Plan: cache only.** The hackathon Vexcel key stays valid until after judging and then
expires, so the live server runs **without** a Vexcel key and serves only what is already in its cache: downtown LA and
Princeton (the areas fetched during the hackathon). Golden's server already does this when it
has no key: map tiles, aerial photos, the 3D mesh and Vexcel heights come from the cache
(cached ~0.9 m height tiles stand in for missing 6 cm ones), and heights fall back to free
USGS LiDAR where nothing is cached. Elsewhere the map has no imagery. (A stricter cache-only
switch with deploy guardrails is golden's task 041, deferred for now; use it once it exists.)

Checklist:
1. **CoStar licence first.** The CoStar data kit (`COSTAR_KIT_DIR`) was given for the
hackathon. Before anything is public, check that its licence allows showing its building
data (names, contacts) on a public site. If it doesn't, run the server without
`COSTAR_KIT_DIR` (building lookups then use golden's fixtures).
2. Where it runs: golden's server on the server PC, next to the other servers, from a folder
like `C:\burvy\golden\` (build it with `cargo build --release -p server` in golden).
3. Cache: copy golden's cache (LA + Princeton; on the dev PC `Z:\3Projects\golden-cache`) to
`C:\burvy\data\golden\cache\` and point `VEXCEL_CACHE_DIR` (and `LIDAR_CACHE_DIR`) at it.
Without a key it can't grow from Vexcel; LiDAR lookups outside the cache still add files, so
keep an eye on the disk.
4. Bind address: `GOLDEN_ADDR` (default `127.0.0.1:8787`). Keep it on `127.0.0.1` if the
reverse proxy runs on the server PC; set it (e.g. `0.0.0.0:8787`, firewall open) only if the
proxy is somewhere else.
5. Reverse proxy: `https://burvy.dev/golden/api/*` to golden's `/api/*` (for example a
Cloudflare Tunnel, or an Origin Rule plus a proxy on the server PC). The address is the
site's own, so golden's server needs no CORS and the iframe's `?api=/golden` stays the same.
6. Keys live on the server PC only, **never** in either repo. Golden reads each key from the
environment, else from its own key file, else from the shared `.env` in `GOLDEN_ENV_FILE`.
A key file can hold just the bare key on one line, e.g. `ANTHROPIC_ENV_FILE` =
`C:\burvy\secrets\golden\anthropic.key`. For cache only: **no** `VEXCEL_API_KEY` and no
`VEXCEL_ENV_FILE` (and none in the shared `.env`). `ANTHROPIC_API_KEY` is optional: without it, golden plans the shot offline (its call sheet says "planned
offline"). `COSTAR_KIT_DIR` (see 1). `GOOGLE_MAPS_API_KEY` only if you want place search.
7. Rate limits: if an Anthropic key is set, every brief costs credit paid by burvy. Add a
per-IP limit on `/golden/api/` (in the proxy, or with Cloudflare rate limiting) before
linking it anywhere, and set a spending limit on the key.
8. Tell visitors what works: the brief should name downtown LA or Princeton (the hero brief
does). Elsewhere there is no imagery.
9. Take "(BACKEND OFFLINE)" off golden's card in `src/experiences/mod.rs`.

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

4. Add a `copy-dir` entry for it in `index.html` (next to the other experiences' entries),
`href="assets/<!name!>"` and `data-target-path="<!name!>"`. **Easy to forget, and skipping it
means trunk never copies the built wasm into `dist/`** - the page 404s on `/<!name!>/<!name!>-wasm.js`
even though everything else (workspace member, build.ps1) is correctly wired.  

5. Do these steps for the UI:
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

- This template is for **Bevy-based** experiences (`game`, `amity`, `floret`), where Bevy's
winit integration looks up an *existing* `<canvas>` element by id.  
For a **raw `winit`+`pixels`** crate (`life-v2`, `venture`), don't create a `<canvas>` at all -
winit creates and appends its own. Use a plain `<div id="game-wrapper"></div>` instead, and set
`canvas_parent: Some("game-wrapper".to_string())` on that crate's `App` (see `life-v2`/`venture`).  
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

# Adding Godot Games
1. Export from godot straight to `assets/`
2. Add a `copy-dir` trunk link to the godot thing in assets
3. `src/experiences/<!name!>.rs`, copy `learn3d.rs`
4. Add the experience in `mod.rs` in `experiences`
5. Route it in `app.rs` in `app/`
