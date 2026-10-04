//! Google sign-in and the settings page, backed by plgroup-server

use gloo_net::http::{Request, RequestBuilder};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::{closure::Closure, JsCast, JsValue};
use plgroup_api::{Me, Session, Settings, SignIn, Subscriber};
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::wasm_bindgen;

/// the OAuth client from Google Cloud project plgroup-510600
const CLIENT_ID: &str = "138340425604-6lu3f36bifqoaa6ikoc7l07ipq51b2jq.apps.googleusercontent.com";
const GIS_SRC: &str = "https://accounts.google.com/gsi/client";
/// localStorage key; a bearer token since cookies from another site get blocked
const TOKEN_KEY: &str = "plgroup-token";

fn api(path: &str) -> String {
    // trunk serve talks to `cargo run --features dev-local`
    let base = if window().location().hostname().as_deref() == Ok("localhost") {
        "http://localhost:3120"
    } else {
        "https://plgroup-api.burvy.dev"
    };
    format!("{base}{path}")
}

// ---- the token

fn storage() -> Option<web_sys::Storage> {
    window().local_storage().ok().flatten()
}

fn token() -> Option<String> {
    storage()?.get_item(TOKEN_KEY).ok().flatten()
}

fn set_token(token: Option<&str>) {
    if let Some(s) = storage() {
        let _ = match token {
            Some(t) => s.set_item(TOKEN_KEY, t),
            None => s.remove_item(TOKEN_KEY),
        };
    }
}

/// adds the signed-in user's token, if any
fn authed(req: RequestBuilder) -> RequestBuilder {
    match token() {
        Some(t) => req.header("Authorization", &format!("Bearer {t}")),
        None => req,
    }
}

/// the server's error text on failure, so it can be shown as is
async fn send(req: Request) -> Result<gloo_net::http::Response, String> {
    let res = req.send().await.map_err(|_| "couldn't reach the server".to_string())?;
    if res.ok() {
        Ok(res)
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

async fn json<T: DeserializeOwned>(req: Request) -> Result<T, String> {
    send(req).await?.json().await.map_err(|e| e.to_string())
}

// ---- who's signed in

/// shared through context by `PLGroup`
#[derive(Clone, Copy)]
pub struct Auth {
    pub me: RwSignal<Option<Me>>,
    /// checking a saved token with the server
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
}

impl Auth {
    pub fn new() -> Auth {
        let auth = Auth {
            me: RwSignal::new(None),
            loading: RwSignal::new(token().is_some()),
            error: RwSignal::new(None),
        };
        if token().is_some() {
            spawn_local(async move {
                let req = authed(Request::get(&api("/me"))).build().unwrap();
                match json::<Me>(req).await {
                    Ok(me) => auth.me.set(Some(me)),
                    // expired or signed out elsewhere
                    Err(_) => set_token(None),
                }
                auth.loading.set(false);
            });
        }
        auth
    }

    async fn sign_in(self, credential: String) {
        let req = Request::post(&api("/auth/google")).json(&SignIn { credential }).unwrap();
        match json::<Session>(req).await {
            Ok(session) => {
                set_token(Some(&session.token));
                self.error.set(None);
                self.me.set(Some(session.me));
            }
            Err(e) => self.error.set(Some(e)),
        }
    }

    async fn sign_out(self) {
        let _ = send(authed(Request::post(&api("/auth/sign-out"))).build().unwrap()).await;
        set_token(None);
        self.me.set(None);
    }
}

// ---- Google's sign-in button (Google Identity Services)

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["google", "accounts", "id"], js_name = initialize)]
    fn gis_initialize(config: &JsValue);
    #[wasm_bindgen(js_namespace = ["google", "accounts", "id"], js_name = renderButton)]
    fn gis_render_button(parent: &JsValue, options: &JsValue);
}

fn js_object(fields: &[(&str, JsValue)]) -> JsValue {
    let obj = js_sys::Object::new();
    for (key, value) in fields {
        let _ = js_sys::Reflect::set(&obj, &(*key).into(), value);
    }
    obj.into()
}

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
fn GoogleButton() -> impl IntoView {
    let auth = expect_context::<Auth>();
    let el = NodeRef::<leptos::html::Div>::new();
    Effect::new(move || {
        let Some(div) = el.get() else { return };
        with_gis(move || {
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

// ---- pages

/// the nav entry: "Sign in", or your name once you are
#[component]
pub fn AccountLink() -> impl IntoView {
    let auth = expect_context::<Auth>();
    let label = move || match auth.me.get() {
        Some(me) => me.name.split_whitespace().next().unwrap_or("Settings").to_string(),
        None => "Sign in".to_string(),
    };
    view! { <leptos_router::components::A href="settings">{label}</leptos_router::components::A> }
}

#[component]
fn Badge() -> impl IntoView {
    view! {
        <span class="plg-badge" title="Signed in with a Penn State account">
            "✓ PSU"
        </span>
    }
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    super::set_title(Some("Settings"));
    let auth = expect_context::<Auth>();
    view! {
        <Show when=move || {
            !auth.loading.get()
        }>
            {move || match auth.me.get() {
                Some(me) => view! { <SignedIn me /> }.into_any(),
                None => {
                    view! {
                        <h2>"Sign in"</h2>
                        <p>
                            "Sign in with Google. (PSU emails get a badge) "
                            <leptos_router::components::A href="privacy">
                                "Privacy"
                            </leptos_router::components::A>
                        </p>
                        <GoogleButton />
                    }
                        .into_any()
                }
            }} {move || auth.error.get().map(|e| view! { <p class="plg-error">{e}</p> })}
        </Show>
    }
}

#[component]
fn SignedIn(me: Me) -> impl IntoView {
    let auth = expect_context::<Auth>();
    let mailing_list = move || auth.me.get().is_some_and(|me| me.settings.mailing_list);

    let set_mailing_list = move |on: bool| {
        auth.me.update(|me| {
            if let Some(me) = me {
                me.settings.mailing_list = on;
            }
        });
        spawn_local(async move {
            let req = authed(Request::put(&api("/me/settings")))
                .json(&Settings { mailing_list: on })
                .unwrap();
            if let Err(e) = send(req).await {
                // put the checkbox back
                auth.me.update(|me| {
                    if let Some(me) = me {
                        me.settings.mailing_list = !on;
                    }
                });
                auth.error.set(Some(e));
            }
        });
    };

    view! {
        <h2>"Settings"</h2>
        <p>
            "Signed in as " {me.name} " (" {me.email} ") "
            {me.psu_verified.then(|| view! { <Badge /> })}
        </p>
        <p>
            <label>
                <input
                    type="checkbox"
                    prop:checked=mailing_list
                    on:change=move |ev| set_mailing_list(event_target_checked(&ev))
                />
                " Put me on the mailing list"
            </label>
        </p>
        <p>
            <button on:click=move |_| spawn_local(auth.sign_out())>"Sign out"</button>
        </p>
        {me.is_admin.then(|| view! { <AdminPanel /> })}
    }
}

// ---- admins only

#[component]
fn AdminPanel() -> impl IntoView {
    let auth = expect_context::<Auth>();
    let subscribers = RwSignal::new(None::<Vec<Subscriber>>);
    let admins = RwSignal::new(Vec::<String>::new());
    let new_admin = RwSignal::new(String::new());

    let report = move |r: Result<(), String>| {
        if let Err(e) = r {
            auth.error.set(Some(e));
        }
    };
    let load_admins = move || {
        spawn_local(async move {
            let req = authed(Request::get(&api("/admin/admins"))).build().unwrap();
            report(json(req).await.map(|list| admins.set(list)));
        })
    };
    load_admins();

    let show_mailing_list = move |_| {
        spawn_local(async move {
            let req = authed(Request::get(&api("/admin/mailing-list"))).build().unwrap();
            report(json(req).await.map(|list| subscribers.set(Some(list))));
        })
    };
    let admin_url = |email: &str| api(&format!("/admin/admins/{}", js_sys::encode_uri_component(email)));
    let add = move |_| {
        let email = new_admin.get_untracked();
        spawn_local(async move {
            let req = authed(Request::put(&admin_url(&email))).build().unwrap();
            report(send(req).await.map(|_| new_admin.set(String::new())));
            load_admins();
        })
    };
    let remove = move |email: String| {
        spawn_local(async move {
            let req = authed(Request::delete(&admin_url(&email))).build().unwrap();
            report(send(req).await.map(|_| ()));
            load_admins();
        })
    };

    view! {
        <h3>"Mailing list"</h3>
        <p>
            <button on:click=show_mailing_list>"Show mailing list"</button>
        </p>
        {move || {
            subscribers
                .get()
                .map(|list| {
                    let emails = list
                        .iter()
                        .map(|s| s.email.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    view! {
                        <p>{list.len()} " on the list. Paste into BCC:"</p>
                        <textarea class="plg-emails" readonly rows="4">
                            {emails}
                        </textarea>
                    }
                })
        }}

        <h3>"Admins"</h3>
        <ul>
            {move || {
                admins
                    .get()
                    .into_iter()
                    .enumerate()
                    .map(|(i, email)| {
                        let button = (i > 0)
                            .then(|| {
                                let email = email.clone();
                                // the first is the owner, who can't be removed
                                view! {
                                    " "
                                    <button on:click=move |_| remove(
                                        email.clone(),
                                    )>"remove"</button>
                                }
                            });
                        view! { <li>{email} {button}</li> }
                    })
                    .collect_view()
            }}
        </ul>
        <p>
            <input type="email" placeholder="email address" bind:value=new_admin />
            " "
            <button on:click=add>"Add admin"</button>
        </p>
    }
}
