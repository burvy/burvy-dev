//! Color Scheme:
//! #002147 #3B2F2F #8B6B4F #D6C7B2 #7A1E2D

use leptos::prelude::*;
use leptos_router::components::{Outlet, A};

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
            <img class="plg-logo" src="/images/plgroup-icon.png" alt="PL Group logo" />
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
