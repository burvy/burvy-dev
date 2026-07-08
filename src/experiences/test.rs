use leptos::prelude::*;

#[component]
pub fn Test() -> impl IntoView {
    let (toggle, set_toggle) = signal(false);
    view! {
        <div>
            <button on:click=move |_| {
                set_toggle.set(!toggle.get())
            }>{move || toggle.get()}</button>
        </div>
    }
}
