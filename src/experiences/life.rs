use leptos::prelude::*;

#[component]
pub fn Life() -> impl IntoView {
    Effect::new(|_| {
        request_animation_frame(|| {
            life_v2::run();
        });
    });

    view! {
        <div
            id="life-canvas"
            style="
            width:800px;
            height:600px;
            overflow:hidden;
            "
        >
            "Loading..."
        </div>
        <p>"space - pause, left click - birth, right click - kill, scroll - change speed"</p>
    }
}
