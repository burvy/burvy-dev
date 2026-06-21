use leptos::prelude::*;
use leptos_router::components::A;

use crate::experiences;

/// renders the experience card, please don't touch this bc it is good
#[component]
pub fn ExperienceCard(experience: experiences::Experience) -> impl IntoView {
    view! {
        <A href=experience.path target="_blank">
            <div class="card" style=format!("background-image: url('{}')", experience.background)>
                <h1>{experience.name}</h1>
                <p>{experience.description}</p>
            </div>
        </A>
    }
}
