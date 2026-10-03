//! Color Scheme:
//! #002147 #3B2F2F #8B6B4F #D6C7B2 #7A1E2D

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes, A};
use leptos_router::hooks::{use_location, use_matched};
use leptos_router::{path, MatchNestedRoutes};
use wasm_bindgen::prelude::wasm_bindgen;

/// images are loaded from burvy.dev even when embedded somewhere else
const ASSETS: &str = "https://burvy.dev";

/// the only place that knows the site lives under /plgroup
#[component(transparent)]
pub fn PLGroupRoutes() -> impl MatchNestedRoutes + Clone {
    view! {
        <ParentRoute path=path!("/plgroup") view=PLGroup>
            <Route path=path!("") view=Home />
            <Route path=path!("about") view=About />
            <Route path=path!("people") view=People />
        </ParentRoute>
    }
    .into_inner()
}

/// entry point for embedding on another page (sites.psu.edu)
#[wasm_bindgen]
pub fn mount(id: &str) {
    let el = document()
        .get_element_by_id(id)
        .expect("no element with that id")
        .unchecked_into();
    leptos::mount::mount_to(el, || {
        view! {
            <Router>
                <Routes fallback=|| view! { <p>"not found"</p> }>
                    <PLGroupRoutes />
                </Routes>
            </Router>
        }
    })
    .forget();
}

#[component]
pub fn PLGroup() -> impl IntoView {
    view! {
        <div class="plgroup">
            <Header />
            <Nav />
            <main class="plg-main">
                <Outlet />
            </main>
        </div>
    }
}

#[component]
fn Header() -> impl IntoView {
    view! {
        <header class="plg-header">
            <img class="plg-logo" src=format!("{ASSETS}/images/plgroup-icon.png") alt="PL Group logo" />
            <div class="plg-title">
                <h1>"Penn State Programming Languages Group"</h1>
            </div>
        </header>
    }
}

#[component]
fn Nav() -> impl IntoView {
    // <A exact> treats WordPress's /plgroup/ (trailing slash) as a different page than /plgroup
    let home = use_matched();
    let pathname = use_location().pathname;
    let home_href = move || {
        let h = home.get();
        if h.is_empty() { "/".to_string() } else { h }
    };
    let at_home = move || pathname.get().trim_end_matches('/') == home.get();
    view! {
        <nav class="plg-nav">
            <a href=home_href aria-current=move || at_home().then_some("page")>
                "Home"
            </a>
            <A href="about">"About"</A>
            <A href="people">"People"</A>
        </nav>
    }
}

#[component]
pub fn Home() -> impl IntoView {
    view! { <h2>"Home"</h2> }
}

#[component]
pub fn About() -> impl IntoView {
    view! { <h2>"About"</h2> }
}

#[component]
pub fn People() -> impl IntoView {
    view! { <h2>"People"</h2> }
}
