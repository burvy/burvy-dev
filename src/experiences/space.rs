use leptos::prelude::*;

#[derive(serde::Deserialize)]
struct Apod {
    url: String, // apod returns a json with a few fields
}

#[component]
pub fn SpacePhotos() -> impl IntoView {
    // LocalResource works better for CSR, it accepts !Send futures.
    // this might be of interest: https://book.leptos.dev/async/10_resources.html
    let apod = LocalResource::new(move || fetch_apod_url());
    view! {
        <div>
            <button>"Click me for a space picture!"</button>
        </div>
        // div puts it on a new line.
        <div>
            // Suspense allows us to use async values that might not be ready yet like apod
            // having the move closure in there also only rerenders the closure not the whole
            // component which is more efficient.
            <Suspense fallback=move || {
                view! { <p>"loading pic..."</p> }
            }>
                // flatten the apod Option<Option<_>> to Option<_>
                {move || apod.get().flatten().map(|url| view! { <img src=url alt="space pic!" /> })}
            </Suspense>
        </div>
    }
}

// this is going to fetch from the network eventually
// but we will use this to fetch from APOD
// where our nasa space images are
async fn fetch_apod_url() -> Option<String> {
    let date = "2026-01-01";
    let key = option_env!("NASA_KEY").unwrap_or("DEMO_KEY"); // this may potentially still be unsafe
    let url = &format!("https://api.nasa.gov/planetary/apod?api_key={}&date={}", key, date);
    // let resp = gloo_net::http::Request::get(url).send().await; // this and .json() may fail
    let apod = gloo_net::http::Request::get(url) // we turn result into option
        .send()
        .await
        .ok()? // some network error
        .json::<Apod>()
        .await
        .ok()?; // some json parse error
    Some(apod.url)

}
