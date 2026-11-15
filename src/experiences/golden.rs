use leptos::prelude::*;

// golden's web page (a copy of golden's `web/`, see the README's "Golden") is a
// static page with its own styles, so it is iframed like metric_balls.rs.
// It is in `/golden-app/`, not `/golden/`: a folder named like the route would
// make a reload of /golden open golden's page on its own instead of this site.
// `?api=/golden` makes it call `/golden/api/...` on this site's address:
// Trunk.toml sends that to golden's server (port 8787) when testing locally, and
// on burvy.dev nginx does the same (README, "Golden: going live"), so the same
// path works on every host and nothing here has to pick by hostname.
#[component]
pub fn Golden() -> impl IntoView {
    view! {
        <div id="game-wrapper">
            <iframe
                src="/golden-app/index.html?api=/golden"
                style="width: 100%; height: 100%; border: none;"
            ></iframe>
        </div>
    }
}
