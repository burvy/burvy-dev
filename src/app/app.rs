use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::pages;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| view! { <pages::notfound::NotFound /> }>
                <Route path=path!("/") view=pages::home::Home />
            </Routes>
        </Router>
    }
}
