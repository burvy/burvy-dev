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
            }>{move || apod.get().map(|url| view! { <img src=url alt="space pic!" /> })}</Suspense>
        </div>
    }
}

// this is going to fetch from the network eventually
// but we will use this to fetch from APOD
// where our nasa space images are
async fn fetch_apod_url() -> String {
    let date = "2026-07-08";
    let key = std::env::var("NASA_KEY").expect("nasa key doesnt exist here");
    let url = &format!("https://api.nasa.gov/planetary/apod?api_key={}&date={}", key, date);
    let resp = gloo_net::http::Request::get(url).send().await;
    todo!()

}
