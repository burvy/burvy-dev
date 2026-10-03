//! Color Scheme:
//! #002147 #3B2F2F #8B6B4F #D6C7B2 #7A1E2D

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes, A};
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
    view! {
        <nav class="plg-nav">
            <A href="" exact=true>
                "Home"
            </A>
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
