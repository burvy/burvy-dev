//! Color Scheme:
//! #002147 #3B2F2F #8B6B4F #D6C7B2 #7A1E2D

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes, A};
use leptos_router::hooks::{use_location, use_matched, use_params_map};
use leptos_router::{path, MatchNestedRoutes};
use serde::Deserialize;
use wasm_bindgen::prelude::wasm_bindgen;

/// images are loaded from burvy.dev even when embedded somewhere else
const ASSETS: &str = "https://burvy.dev";

/// page content lives in WordPress; this is its REST API
const WP: &str = "https://sites.psu.edu/plgroup/wp-json/wp/v2";

const SITE_NAME: &str = "Programming Languages @ PSU";

/// the only place that knows the site lives under /plgroup
#[component(transparent)]
pub fn PLGroupRoutes() -> impl MatchNestedRoutes + Clone {
    view! {
        <ParentRoute path=path!("/plgroup") view=PLGroup>
            <Route path=path!("") view=|| view! { <WpPage slug="home" /> } />
            <Route path=path!(":slug") view=SlugPage />
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

#[derive(Clone, Deserialize)]
struct Page {
    slug: String,
    title: Rendered,
    content: Rendered,
    menu_order: i32,
}

/// WordPress sends titles and content as HTML
#[derive(Clone, Deserialize)]
struct Rendered {
    rendered: String,
}

type Pages = LocalResource<Result<Vec<Page>, String>>;

/// every published page: home first, then by the "Order" field in WordPress,
/// then oldest first
async fn fetch_pages() -> Result<Vec<Page>, String> {
    let url = format!("{WP}/pages?per_page=100&orderby=date&order=asc&_fields=slug,title,content,menu_order");
    let mut pages: Vec<Page> = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    pages.sort_by_key(|p| (p.slug != "home", p.menu_order));
    Ok(pages)
}

#[component]
pub fn PLGroup() -> impl IntoView {
    // fetched once; every page and the nav read from it
    let pages: Pages = LocalResource::new(fetch_pages);
    provide_context(pages);
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
    let pages = expect_context::<Pages>();
    // <A exact> treats WordPress's /plgroup/ (trailing slash) as a different page than /plgroup
    let home = use_matched();
    let pathname = use_location().pathname;
    let home_href = move || {
        let h = home.get();
        if h.is_empty() { "/".to_string() } else { h }
    };
    let at_home = move || pathname.get().trim_end_matches('/') == home.get();
    let links = move || {
        let pages = pages.get().and_then(Result::ok).unwrap_or_default();
        pages
            .into_iter()
            .map(|p| {
                let title = view! { <span inner_html=p.title.rendered></span> };
                if p.slug == "home" {
                    view! {
                        <a href=home_href aria-current=move || at_home().then_some("page")>
                            {title}
                        </a>
                    }
                    .into_any()
                } else {
                    view! { <A href=p.slug>{title}</A> }.into_any()
                }
            })
            .collect_view()
    };
    view! { <nav class="plg-nav">{links}</nav> }
}

#[component]
fn SlugPage() -> impl IntoView {
    let params = use_params_map();
    let slug = Signal::derive(move || params.read().get("slug").unwrap_or_default());
    view! { <WpPage slug /> }
}

/// one WordPress page: its title and whatever was written in the editor
#[component]
fn WpPage(#[prop(into)] slug: Signal<String>) -> impl IntoView {
    let pages = expect_context::<Pages>();
    let page = move || {
        let slug = slug.get();
        pages.get().map(|r| r.map(|ps| ps.into_iter().find(|p| p.slug == slug)))
    };

    // client-side navigation doesn't reload, so WordPress can't update the tab title
    Effect::new(move || {
        if let Some(Ok(Some(p))) = page() {
            let title = if p.slug == "home" {
                SITE_NAME.to_string()
            } else {
                format!("{} – {SITE_NAME}", html_to_text(&p.title.rendered))
            };
            document().set_title(&title);
        }
    });

    move || match page() {
        None => view! { <p class="plg-loading">"Loading…"</p> }.into_any(),
        Some(Err(e)) => view! { <p>"Couldn't load this page: " {e}</p> }.into_any(),
        Some(Ok(None)) => view! { <h2>"Page not found"</h2> }.into_any(),
        Some(Ok(Some(p))) => view! {
            <h2 inner_html=p.title.rendered></h2>
            <div class="plg-content" inner_html=p.content.rendered></div>
        }
        .into_any(),
    }
}

/// "Q&amp;A" -> "Q&A"
fn html_to_text(html: &str) -> String {
    let el = document().create_element("div").unwrap();
    el.set_inner_html(html);
    el.text_content().unwrap_or_default()
}
