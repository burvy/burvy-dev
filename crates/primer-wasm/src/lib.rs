use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// Entry point the site calls after dynamically importing this module.
/// Primer is built inside a `<div>` of its own (the "host"), which is kept in
/// `window.primerHost`. The site's /primer page puts the host into the page;
/// coming back to the page later reuses it, so a signed-in reader stays so.
#[wasm_bindgen]
pub fn start() {
    let document = web_sys::window().unwrap().document().unwrap();
    let host: web_sys::HtmlElement = document.create_element("div").unwrap().unchecked_into();
    // Picked the way plgroup picks its server: on localhost the server is
    // reached through the site's own address (/api/...), which trunk proxies
    // to port 3140 (see Trunk.toml); anywhere else it is the live server.
    // The drawings and sounds are copied next to this module (see index.html).
    let local = web_sys::window().unwrap().location().hostname().as_deref() == Ok("localhost");
    let config = primer_web::Config {
        api: if local { String::new() } else { "https://primer-api.burvy.dev".into() },
        assets: "/primer/assets".into(),
    };
    primer_web::mount(host.clone(), config);
    let _ = js_sys::Reflect::set(&js_sys::global(), &"primerHost".into(), &host);
    if let Some(page) = document.get_element_by_id("primer-wrapper") {
        let _ = page.append_child(&host);
    }
}
