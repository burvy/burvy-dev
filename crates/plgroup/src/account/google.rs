//! Google's sign-in button (Google Identity Services)

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen::prelude::wasm_bindgen;

use super::Auth;

/// the OAuth client from Google Cloud project plgroup-510600
const CLIENT_ID: &str = "138340425604-6lu3f36bifqoaa6ikoc7l07ipq51b2jq.apps.googleusercontent.com";
const GIS_SRC: &str = "https://accounts.google.com/gsi/client";

// the functions Google's script defines, callable from Rust once it has loaded
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["google", "accounts", "id"], js_name = initialize)]
    fn gis_initialize(config: &JsValue);
    #[wasm_bindgen(js_namespace = ["google", "accounts", "id"], js_name = renderButton)]
    fn gis_render_button(parent: &JsValue, options: &JsValue);
}

/// a JavaScript `{ key: value, ... }` object
fn js_object(fields: &[(&str, JsValue)]) -> JsValue {
    let obj = js_sys::Object::new();
    for (key, value) in fields {
        let _ = js_sys::Reflect::set(&obj, &(*key).into(), value);
    }
    obj.into()
}

/// whether `google.accounts` exists yet
fn gis_ready() -> bool {
    let get = |obj: &JsValue, key: &str| js_sys::Reflect::get(obj, &key.into()).unwrap_or_default();
    !get(&get(&window(), "google"), "accounts").is_undefined()
}

/// loads Google's script on first use, then runs `f`
fn with_gis(f: impl FnOnce() + 'static) {
    if gis_ready() {
        return f();
    }
    let doc = document();
    let script = doc.get_element_by_id("plg-gis").unwrap_or_else(|| {
        let s = doc.create_element("script").unwrap();
        s.set_id("plg-gis");
        s.set_attribute("src", GIS_SRC).unwrap();
        doc.head().unwrap().append_child(&s).unwrap();
        s
    });
    let onload = Closure::once_into_js(f);
    script.add_event_listener_with_callback("load", onload.unchecked_ref()).unwrap();
}

#[component]
pub fn GoogleButton() -> impl IntoView {
    let auth = expect_context::<Auth>();
    let el = NodeRef::<leptos::html::Div>::new();
    Effect::new(move || {
        let Some(div) = el.get() else { return };
        with_gis(move || {
            // Google calls this with the signed note once someone picks an account
            let callback = Closure::<dyn Fn(JsValue)>::new(move |res: JsValue| {
                let credential = js_sys::Reflect::get(&res, &"credential".into())
                    .ok()
                    .and_then(|c| c.as_string());
                if let Some(credential) = credential {
                    spawn_local(auth.sign_in(credential));
                }
            });
            gis_initialize(&js_object(&[
                ("client_id", CLIENT_ID.into()),
                ("callback", callback.into_js_value()),
            ]));
            gis_render_button(
                &div,
                &js_object(&[
                    ("theme", "outline".into()),
                    ("size", "large".into()),
                    ("text", "signin_with".into()),
                ]),
            );
        });
    });
    view! { <div class="plg-google" node_ref=el></div> }
}
