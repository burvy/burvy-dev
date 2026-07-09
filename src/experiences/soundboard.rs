use super::super::components::sound;
use leptos::prelude::*;
use leptos_router::components::A;

/// a list of all the sounds
/// just increment the list size and add more
/// sounds onto it
const SOUNDS: [&str; 2] = ["sounds/ding.mp3", "sounds/question.mp3"];

#[component]
pub fn Soundboard() -> impl IntoView {
    view! { <p>"test"</p> }
}

/// repurposing the experience card to make sound cards
/// TODO: keep going
#[component]
pub fn SoundCard(sound: &'static str) -> impl IntoView {
    let play_sound = sound::play_sound(sound);
    view! {
        <div
            // instance 2 of using the sound::play_sound api
            // just play the sound directly
            on:click=move |_| {
                play_sound();
            }
            class="card"
        >
            <h1>{sound}</h1>
        </div>
    }
}
