use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::lazy;

/// Primer's files, copied by `crates/primer-wasm/index.html`.
const SCRIPTS: [&str; 3] = ["audio.js", "art.js", "signin.js"];

/// Primer's two stylesheets are written for a page of their own (they style
/// `body`), so they are only on the page while this experience is open.
/// They point at `../../assets/...`, which is `/assets` from the site root;
/// here the assets live in `/primer/assets`, so the address is rewritten.
async fn add_styles() {
    // The site styles every element (`* { font-family: Helvetica }`), which would
    // beat the font Primer's body passes down. This rule has no weight of its own
    // (`:where`), so it only wins over that and nothing of Primer's.
    let mut css = String::from(":where(#primer-wrapper) * { font-family: var(--body-font); }");
    for file in ["style.css", "play.css"] {
        if let Ok(response) = gloo_net::http::Request::get(&format!("/primer/{file}")).send().await {
            css += &response.text().await.unwrap_or_default();
        }
    }
    let style = document().create_element("style").unwrap();
    style.set_id("primer-styles");
    style.set_text_content(Some(&css.replace("../../assets/", "/primer/assets/")));
    let _ = document().head().unwrap().append_child(&style);
}

#[component]
pub fn Primer() -> impl IntoView {
    let page = NodeRef::<leptos::html::Div>::new();

    Effect::new(move |_| {
        let Some(page) = page.get() else { return };
        for file in SCRIPTS {
            let id = format!("primer-{file}");
            if document().get_element_by_id(&id).is_none() {
                let script = document().create_element("script").unwrap();
                script.set_id(&id);
                let _ = script.set_attribute("src", &format!("/primer/{file}"));
                let _ = document().head().unwrap().append_child(&script);
            }
        }
        leptos::task::spawn_local(async move {
            add_styles().await;
            // Visited before: Primer is already running, just put it back.
            // (`primerHost` is set by `crates/primer-wasm`.)
            match js_sys::Reflect::get(&js_sys::global(), &"primerHost".into()) {
                Ok(host) if host.is_object() => {
                    let _ = page.append_child(host.unchecked_ref());
                }
                _ => lazy::start_experience("/primer/primer-wasm.js"),
            }
        });
    });

    on_cleanup(|| {
        if let Some(style) = document().get_element_by_id("primer-styles") {
            style.remove();
        }
    });

    view! { <div id="primer-wrapper" node_ref=page></div> }
}
