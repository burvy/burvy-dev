use super::super::components::sound;
use leptos::prelude::*;

/// a list of all the sounds
/// just increment the list size and add more
/// sounds onto it
const SOUNDS: [&str; 3] = [
    "sounds/ding.mp3",
    "sounds/question.mp3",
    "sounds/aria-math.mp3",
];

#[component]
pub fn Soundboard() -> impl IntoView {
    view! {
        <p>
            "test"
            {SOUNDS
                .iter()
                .copied()
                .map(|path| {
                    view! { <SoundCard sound=path /> }
                })
                .collect::<Vec<_>>()}
        </p>
    }
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
