//! admins only: the mailing list, and who else is an admin

use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;
use plgroup_api::Subscriber;

use super::Auth;
use super::api::{api, authed, json, send};

#[component]
pub fn AdminPanel() -> impl IntoView {
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

    let toggle_mailing_list = move |_| {
        if subscribers.with_untracked(|s| s.is_some()) {
            subscribers.set(None);
        } else {
            spawn_local(async move {
                let req = authed(Request::get(&api("/admin/mailing-list")))
                    .build()
                    .unwrap();
                report(json(req).await.map(|list| subscribers.set(Some(list))));
            })
        }
    };
    let admin_url = |email: &str| {
        api(&format!(
            "/admin/admins/{}",
            js_sys::encode_uri_component(email)
        ))
    };
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
            <button on:click=toggle_mailing_list>
                {move || {
                    if subscribers.get().is_some() {
                        "Hide mailing list"
                    } else {
                        "Show mailing list"
                    }
                }}
            </button>
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
