//! Google sign-in and the settings page, backed by plgroup-server

mod admin;
mod api;
mod google;
mod settings;

use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use plgroup_api::{Me, Session, SignIn};

use api::{api, authed, json, send, set_token, token};
pub use settings::SettingsPage;

/// who's signed in, shared through context by `PLGroup`
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

    /// trades Google's credential for our own token
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

/// the nav entry: "Sign in", or your name once you are
#[component]
pub fn AccountLink() -> impl IntoView {
    let auth = expect_context::<Auth>();
    let label = move || match auth.me.get() {
        Some(me) => me.name.split_whitespace().next().unwrap_or("Settings").to_string(),
        None => "Sign in".to_string(),
    };
    view! { <A href="settings">{label}</A> }
}
