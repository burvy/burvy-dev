use leptos::prelude::*;

use crate::components::ExperienceCard;
use crate::experiences;

/// displays the homepage, go to `app.rs` and `experiences/` to add new things
#[component]
pub fn Home() -> impl IntoView {
    view! {
        <h1>"hi welcome to my site"</h1>
        <section class="card-list">
            // newest elements at the bottom are the newest at the top
            {experiences::EXPERIENCES
                .iter()
                .copied()
                .rev()
                .map(|exp| {
                    view! { <ExperienceCard experience=exp /> }
                })
                .collect::<Vec<_>>()}
        </section>
    }
}
