use leptos::prelude::*;

use crate::set_title;

#[component]
pub fn People() -> impl IntoView {
    set_title(Some("People"));
    view! {
        <h2>"People"</h2>
        <p>"Content Pending"</p>
    }
}
