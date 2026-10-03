//! Color Scheme:
//! #002147 #3B2F2F #8B6B4F #D6C7B2 #7A1E2D

use leptos::prelude::*;

#[component]
pub fn PLGroup() -> impl IntoView {
    view! {
        <div class="plgroup">
            <Header />
            <Nav />
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
            <a href="#">"Home"</a>
            <a href="#">"About"</a>
            <a href="#">"People"</a>
        </nav>
    }
}
