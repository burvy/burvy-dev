use leptos::prelude::*;

#[component]
pub fn Test() -> impl IntoView {
    let (toggle, set_toggle) = signal(false);
    // TODO: make these work
    let (x, set_x) = signal(20_u32);
    let (y, set_y) = signal(300_u32);
    view! {
        <div>
            <button on:click=move |_| {
                set_toggle.set(!toggle.get())
            }>{move || toggle.get()}</button>
        </div>
        <div
            class="player"
            style:left=format!("left: {}", x.get())
            style:top=format!("top: {}", y.get())
        ></div>
    }
}
