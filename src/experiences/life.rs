use leptos::prelude::*;

#[component]
pub fn Life() -> impl IntoView {
    Effect::new(move |_| {
        request_animation_frame(move || {
            life_v2::run();
        });
    });
    view! {
        <div id="life-canvas"></div>
        <p>"space - pause, left click - birth, right click - kill, scroll - change speed"</p>
    }
}
