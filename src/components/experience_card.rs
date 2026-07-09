use super::super::components::sound;
use leptos::prelude::*;
use leptos_router::components::A;

use crate::experiences;

/// renders the experience card, please don't touch this bc it is good
#[component]
pub fn ExperienceCard(experience: experiences::Experience) -> impl IntoView {
    let play_ques = sound::play_sound("/sounds/question.mp3");
    view! {
        // TODO: make sure theres a way to get back to the homepage later
        <A href=experience.path>
            <div
                // instance 2 of using the sound::play_sound api
                // just play the sound directly
                on:click=move |_| {
                    play_ques();
                }
                class="card"
                style=format!("background-image: url('{}')", experience.background)
            >
                <h1>{experience.name}</h1>
                <p>{experience.description}</p>
            </div>
        </A>
    }
}
