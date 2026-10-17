use leptos::prelude::*;

#[derive(Clone)]
struct Video {
    label: String,
    id: i32,
}
#[component]
pub fn VideoSection() -> impl IntoView {
    let videos = video_choices();
    let (id, set_id) = signal(videos[0].id);
    let (url, set_url) = signal(video_url_for(videos[0].id));

    view! {
        <div class="child body-child body-main">
            <VideoPlayer url=url />
            <div class="body-two">
                <div id="main-button-info">"Click one of these buttons below!"</div>
                <VideoButtons videos=videos selected_id=id set_selected_id=set_id set_url=set_url />
            </div>
        </div>
    }
}

#[component]
fn VideoPlayer(url: ReadSignal<&'static str>) -> impl IntoView {
    view! {
        <iframe
            width="560"
            height="315"
            src=move || url.get()
            title="YouTube video player"
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
            referrerpolicy="strict-origin-when-cross-origin"
            allowfullscreen
            class="body-two"
        ></iframe>
    }
}

#[component]
fn VideoButtons(
    videos: Vec<Video>,
    selected_id: ReadSignal<i32>,
    set_selected_id: WriteSignal<i32>,
    set_url: WriteSignal<&'static str>,
) -> impl IntoView {
    view! {
        <div class="body-child">
            {videos
                .into_iter()
                .map(|video| {
                    view! {
                        <button
                            on:click=move |_| {
                                set_selected_id.set(video.id);
                                set_url.set(video_url_for(video.id));
                            }
                            class="main-button"
                            class:active=move || selected_id.get() == video.id
                            class:inactive=move || selected_id.get() != video.id
                        >
                            {video.label}
                        </button>
                    }
                })
                .collect::<Vec<_>>()} <p>{selected_id}</p>
        </div>
    }
}

fn video_choices() -> Vec<Video> {
    let mut output = vec![];
    for i in 1..=12 {
        output.push({
            Video {
                id: i,
                label: format!("Video {}", i),
            }
        });
    }
    output
}

/// Match an ID to a video URL.
fn video_url_for(id: i32) -> &'static str {
    match id {
        1 => "https://www.youtube.com/embed/ivGK0sNju5s?si=C5KL8jHFUjcQd1U-",
        2 => "https://www.youtube.com/embed/CdYtavp2h24?si=oBFzoC4V169_UVxp",
        3 => "https://www.youtube.com/embed/Lcupxa72cLk?si=s2IgYDHDwJKvt4rB",
        4 => "https://www.youtube.com/embed/cqd00g20bkI?si=znIN52dyM1A07JH9",
        5 => "https://www.youtube.com/embed/HlnNSW_Yi70?si=krmzKolIwR7EZOAq",
        6 => "https://www.youtube.com/embed/wcRfV6ahkh0?si=LeE3M3i8_Va-XDAl",
        7 => "https://www.youtube.com/embed/0iSz6aDAZBA?si=uBRqGe-sy0E4_X-o",
        8 => "https://www.youtube.com/embed/CdYtavp2h24?si=oBFzoC4V169_UVxp",
        9 => "https://www.youtube.com/embed/Lcupxa72cLk?si=s2IgYDHDwJKvt4rB",
        10 => "https://www.youtube.com/embed/cqd00g20bkI?si=znIN52dyM1A07JH9",
        11 => "https://www.youtube.com/embed/HlnNSW_Yi70?si=krmzKolIwR7EZOAq",
        12 => "https://www.youtube.com/embed/wcRfV6ahkh0?si=LeE3M3i8_Va-XDAl",
        _ => "https://www.youtube.com/embed/3ZTdoWPDiOc?si=33aJbEe2FcbV5Fga",
    }
}
