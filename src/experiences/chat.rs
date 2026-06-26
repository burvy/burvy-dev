use leptos::prelude::*;
use leptos::html;
use wasm_bindgen_futures::spawn_local;
use crate::webtrans::{WebTrans, connect};

#[component]
pub fn Chat() -> impl IntoView {
    let (input, set_input) = signal(String::new());
    let (messages, set_messages) = signal(Vec::<String>::new());
    let (status, set_status) = signal("connecting".to_string());
    let (client, set_client) = signal(None::<WebTrans>);

    spawn_local(async move {
        match connect("/chat", move |message| {
            set_messages.update(|messages| messages.push(message));
        })
        .await
        {
            Ok(next_client) => {
                set_client.set(Some(next_client));
                set_status.set("connected".to_string());
            }
            Err(_) => {
                set_status.set("disconnected".to_string());
            }
        }
    });

    on_cleanup(move || {
        if let Some(client) = client.get_untracked() {
            client.close();
        }
    });

    let send = move || {
        // no just spaces as well
        let message = input.get().trim().to_string();

        // mhm
        if message.is_empty() {
            return;
        }

        // we must be a tracked client
        let Some(client) = client.get_untracked() else {
            set_status.set("not connected".to_string());
            return;
        };

        // clear input after sent
        set_input.set(String::new());

        // send the message and check if it is an error at the same time
        spawn_local(async move {
            if client.send(&message).await.is_err() {
                set_status.set("send failed".to_string());
            }
        });
    };
    let list_ref = NodeRef::<html::Ul>::new();
    Effect::new(move |_| {
        messages.track(); // track changes
        if let Some(ul) = list_ref.get() {
            request_animation_frame(move || {
                ul.set_scroll_top(ul.scroll_height() as f64); // scroll to bottom when change
            });
        }
    });
    view! {
        <main class="chat-page">
            <h1>"Chatroom"</h1>

            <p>"you are " {move || status.get()}</p>
            <div>
                <ul class="chat-messages" node_ref=list_ref>
                    {move || {
                        messages
                            .get()
                            .into_iter()
                            .map(|message| view! { <li>{message}</li> })
                            .collect_view()
                    }}
                </ul>
            </div>

            <form on:submit=move |ev| {
                ev.prevent_default();
                send();
            }>
                <div class="texterior">
                    <input
                        class="tinput"
                        placeholder="type something!"
                        on:input:target=move |ev| set_input.set(ev.target().value())
                        prop:value=input
                    />

                    <button
                        class="submit"
                        type="submit"
                        disabled=move || status.get() != "connected"
                    >
                        ">>"
                    </button>
                </div>
            </form>
        </main>
    }
}
