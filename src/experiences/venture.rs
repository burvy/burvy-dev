use leptos::prelude::*;

use crate::lazy;

#[component]
pub fn Venture() -> impl IntoView {
    let canvas = NodeRef::<leptos::html::Canvas>::new();

    Effect::new(|_| {
        request_animation_frame(|| {
            lazy::start_experience("/venture/venture-wasm.js");
        });
    });

    view! {
        <div id="game-wrapper"></div>
    }
}
