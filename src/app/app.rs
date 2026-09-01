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
                // TODO: change this to possibly use a macro,
                // not sure if it is possible within the view macro
                // do a bit of research
                <Route path=path!("/space-photos") view=experiences::space::SpacePhotos />
                <Route path=path!("/soundboard") view=experiences::soundboard::Soundboard />
                <Route path=path!("/life") view=experiences::life::Life />
                <Route path=path!("/game") view=experiences::game::Game />
                <Route path=path!("/planner") view=experiences::planner::Planner />
            </Routes>
        </Router>
    }
}
