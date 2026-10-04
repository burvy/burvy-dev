//! Color Scheme:
//! #002147 #3B2F2F #8B6B4F #D6C7B2 #7A1E2D

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes, A};
use leptos_router::hooks::{use_location, use_matched};
use leptos_router::{path, MatchNestedRoutes};
use wasm_bindgen::prelude::wasm_bindgen;

mod account;

/// images are loaded from burvy.dev even when embedded somewhere else
const ASSETS: &str = "https://burvy.dev";

const SITE_NAME: &str = "Programming Languages @ PSU";

/// the only place that knows the site lives under /plgroup
#[component(transparent)]
pub fn PLGroupRoutes() -> impl MatchNestedRoutes + Clone {
    view! {
        <ParentRoute path=path!("/plgroup") view=PLGroup>
            <Route path=path!("") view=Home />
            <Route path=path!("about") view=About />
            <Route path=path!("people") view=People />
            <Route path=path!("links") view=Links />
            <Route path=path!("settings") view=account::SettingsPage />
            <Route path=path!("privacy") view=Privacy />
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
    provide_context(account::Auth::new());
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
            <img
                class="plg-logo"
                src=format!("{ASSETS}/images/plgroup-icon.png")
                alt="PL Group logo"
            />
            <div class="plg-title">
                <h1>"Programming Languages Group at Penn State"</h1>
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
        if h.is_empty() {
            "/".to_string()
        } else {
            h
        }
    };
    let at_home = move || pathname.get().trim_end_matches('/') == home.get();
    view! {
        <nav class="plg-nav">
            <a href=home_href aria-current=move || at_home().then_some("page")>
                "Home"
            </a>
            <A href="about">"About"</A>
            <A href="people">"People"</A>
            <A href="links">"Links"</A>
            <account::AccountLink />
            <A href="privacy">"Privacy"</A>
        </nav>
    }
}

/// client-side navigation doesn't reload, so the host page can't update the tab title
fn set_title(page: Option<&str>) {
    let title = match page {
        Some(page) => format!("{page} – {SITE_NAME}"),
        None => SITE_NAME.to_string(),
    };
    document().set_title(&title);
}

#[component]
pub fn Home() -> impl IntoView {
    set_title(None);
    view! {
        <h2>"Home"</h2>
        <p>
            "Do you like math, programming, or both? " <A href="links">"Join us!"</A>
            " Welcome to the Programming Languages Group at Penn State!"
        </p>
    }
}

#[component]
pub fn About() -> impl IntoView {
    set_title(Some("About"));
    view! { <h2>"About"</h2> }
}

#[component]
pub fn People() -> impl IntoView {
    set_title(Some("People"));
    view! { <h2>"People"</h2> }
}

#[component]
pub fn Links() -> impl IntoView {
    set_title(Some("Links"));
    view! {
        <h2>"Links"</h2>
        <p>
            <a href="https://discord.gg/Xp3tpk9cvP">"Discord Server"</a>
        </p>
    }
}

#[component]
pub fn Privacy() -> impl IntoView {
    set_title(Some("Privacy"));
    view! {
        <h2>"Privacy"</h2>
        <p>
            "When you sign in with Google, we store your name, your email address, "
            "whether or not it is a Penn State account, and whether or not you joined the mailing list."
        </p>
        <p>
            "We use this only to sign you in, identify content you post here, "
            "and to send club emails if you joined the " "mailing list. We never share or sell it."
        </p>
        <p>
            "You can leave the mailing list at any time in " <A href="../settings">"Settings"</A>
            ". To have your data deleted, email me at "
            <a href="mailto:bql5601@psu.edu">"bql5601@psu.edu"</a> "."
        </p>
    }
}
