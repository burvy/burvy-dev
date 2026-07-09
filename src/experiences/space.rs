use leptos::prelude::*;

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
    "https://apod.nasa.gov/apod/image/2607/NGC6769LRGBcropAZ-1500-20-May-2026.jpg".to_string()
}
