use super::super::components::sound;
use leptos::prelude::*;

/// a list of all the sounds
/// just increment the list size and add more
/// sounds onto it
const SOUNDS: [SoundInfo; 3] = [
    SoundInfo {
        path: "sounds/ding.mp3",
        name: "🔔",
        background: "images/ding.png",
    },
    SoundInfo {
        path: "sounds/question.mp3",
        name: "❓",
        background: "images/question.png",
    },
    SoundInfo {
        path: "sounds/aria-math.mp3",
        name: "Old",
        background: "images/aria-math.png",
    },
];

#[derive(Copy, Clone)]
pub struct SoundInfo {
    pub path: &'static str,
    pub name: &'static str,
    pub background: &'static str,
}

#[component]
pub fn Soundboard() -> impl IntoView {
    view! {
        <p class="card-list">
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
pub fn SoundCard(sound: SoundInfo) -> impl IntoView {
    let play_sound = sound::play_sound(sound.path);
    view! {
        <div
            // instance 2 of using the sound::play_sound api
            // just play the sound directly
            on:click=move |_| {
                play_sound();
            }
            class="card"
            style=format!("background-image: url('{}')", sound.background)
        >
            <h1>{sound.name}</h1>
        </div>
    }
}
