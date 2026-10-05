//! talking to plgroup-server: the saved token, and sending requests

use gloo_net::http::{Request, RequestBuilder, Response};
use leptos::prelude::*;
use serde::de::DeserializeOwned;

/// localStorage key; a bearer token since cookies from another site get blocked
const TOKEN_KEY: &str = "plgroup-token";

/// the full URL for a server path like "/me"
pub fn api(path: &str) -> String {
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

pub fn token() -> Option<String> {
    storage()?.get_item(TOKEN_KEY).ok().flatten()
}

pub fn set_token(token: Option<&str>) {
    if let Some(s) = storage() {
        let _ = match token {
            Some(t) => s.set_item(TOKEN_KEY, t),
            None => s.remove_item(TOKEN_KEY),
        };
    }
}

/// adds the signed-in user's token, if any
pub fn authed(req: RequestBuilder) -> RequestBuilder {
    match token() {
        Some(t) => req.header("Authorization", &format!("Bearer {t}")),
        None => req,
    }
}

// ---- sending

/// the server's error text on failure, so it can be shown as is
pub async fn send(req: Request) -> Result<Response, String> {
    let res = req.send().await.map_err(|_| "couldn't reach the server".to_string())?;
    if res.ok() {
        Ok(res)
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// `send`, then read the reply as `T`
pub async fn json<T: DeserializeOwned>(req: Request) -> Result<T, String> {
    send(req).await?.json().await.map_err(|e| e.to_string())
}
