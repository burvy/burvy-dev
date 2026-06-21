use leptos::prelude::*;

use crate::components::ExperienceCard;
use crate::experiences;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <div>
            {experiences::EXPERIENCES
                .iter()
                .copied()
                .map(|exp| view! { <ExperienceCard experience=exp /> })
                .collect::<Vec<_>>()}
        </div>
    }
}
