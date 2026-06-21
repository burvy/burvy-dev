use leptos::prelude::*;

#[component]
pub fn Chat() -> impl IntoView {
    let (input, set_input) = signal("".to_string());
    view! {
        <textarea
            class="text-input"
            placeholder="type something!"
            on:input:target=move |ev| { set_input.set(ev.target().value()) }
            prop:value=input
        />
        <p>{input}</p>
    }
}
