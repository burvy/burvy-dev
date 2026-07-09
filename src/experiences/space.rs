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
