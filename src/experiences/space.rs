use leptos::prelude::*;

#[component]
pub fn SpacePhotos() -> impl IntoView {
    view! {
        <div>
            <button>"Click me for a space picture!"</button>
        </div>
        // div puts it on a new line.
        <div>
            <img
                src="https://apod.nasa.gov/apod/image/2607/NGC6769LRGBcropAZ-1500-20-May-2026-1024.jpg"
                alt="image description"
            />
        </div>
    }
}

// this is going to fetch from the network eventually
// but we will use this to fetch from APOD
// where our nasa space images are
async fn fetch_apod_url() -> String {
    "https://apod.nasa.gov/apod/image/2607/NGC6769LRGBcropAZ-1500-20-May-2026.jpg".to_string()
}
