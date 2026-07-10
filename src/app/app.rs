use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::experiences;
use crate::pages;

/// this is the main hub for new sites
/// add a new site with a new Route
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| view! { <pages::notfound::NotFound /> }>
                <Route path=path!("/") view=pages::home::Home />
                <Route path=path!("/test") view=experiences::test::Test />
                <Route path=path!("/chat") view=experiences::chat::Chat />
                // TODO: change this
                <Route path=path!("/space-photos") view=experiences::space::SpacePhotos />
                <Route path=path!("/soundboard") view=experiences::soundboard::Soundboard />
                <Route path=path!("/life") view=experiences::life::Life />
            </Routes>
        </Router>
    }
}
