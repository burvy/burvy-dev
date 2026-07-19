use leptos::prelude::*;

#[component]
pub fn Game() -> impl IntoView {
    let canvas = NodeRef::<leptos::html::Canvas>::new();

    Effect::new(move |_| {
        if let Some(_) = canvas.get() {
            web_sys::console::log_1(&"Canvas exists!".into());
            game::run();
        }
    });

    view! {
        <div id="game-wrapper">
            <canvas node_ref=canvas id="game-canvas">
                "Loading..."
            </canvas>
        </div>
    }
}
