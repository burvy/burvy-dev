//! the Settings page: sign in, or your settings once you are

use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use plgroup_api::{Me, Settings};

use super::admin::AdminPanel;
use super::api::{api, authed, send};
use super::google::GoogleButton;
use super::Auth;

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
    crate::set_title(Some("Settings"));
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
                        <p>"Sign in with Google. (PSU emails get a badge) "</p>
                        <p>
                            <A href="../privacy">"Privacy"</A>
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
