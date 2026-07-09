use leptos::prelude::*;

#[component]
pub fn SpacePhotos() -> impl IntoView {
    view! {
        <button>"Click me for a space picture!"</button>
        <img
            src="https://apod.nasa.gov/apod/image/2607/NGC6769LRGBcropAZ-1500-20-May-2026-1024.jpg"
            alt="image description"
        />
    }
}
