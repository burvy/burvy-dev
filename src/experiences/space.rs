use leptos::prelude::*;
use js_sys::{Date, Math};

#[derive(serde::Deserialize)]
struct Apod {
    url: String, // apod returns a json with a few fields
}

#[component]
pub fn SpacePhotos() -> impl IntoView {
    let boom_sfx = "sounds/boom.mp3";
    let play_boom = move || {
        let _ = web_sys::HtmlAudioElement::new_with_src(boom_sfx)
            .and_then(|audio| audio.play());
    };
    Effect::new(move |_| {play_boom();});
    // a value that just changes
    let (reload, set_reload) = signal(false); // arbritrary, just needs to change
    // LocalResource works better for CSR, it accepts !Send futures.
    // this might be of interest: https://book.leptos.dev/async/10_resources.html
    let apod = LocalResource::new(move || {
        reload.get(); // changing this re runs the closure, updating the image
        fetch_apod_url()});
    view! {
        <div>
            // just change the value of reload, any change triggers image reload
            // and_then consumes audio and plays it directly
            <button on:click=move |_| {
                let _ = play_boom;
                set_reload.update(|n| *n = !*n);
            }>"Click me for a space picture!"</button>
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
    let date = random_apod_date(); // must be formatted like YYYY-MM-DD
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

/// Random date from the start of APOD to now
/// formatted like YYYY-MM-DD (no extra stuff at the end)
fn random_apod_date() -> String {
    let start = Date::parse("1995-06-16"); // APOD's first day! https://en.wikipedia.org/wiki/Astronomy_Picture_of_the_Day
    let now = Date::now();
    let random_date = start + Math::random() * (now - start); // obvious math... random date
    let iso = Date::new(&random_date.into()).to_iso_string(); // YYYY-MM-DD--...
    iso.as_string().unwrap_or("1995-06-16".to_string())[..10].to_string()

}
