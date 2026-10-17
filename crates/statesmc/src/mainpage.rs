use crate::elements;
use leptos::prelude::*;
use leptos_router::components::*;
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Header />
        <Body />
        <MapSection />
    }
}

#[component]
fn Header() -> impl IntoView {
    view! {
        <div class="header parent">
            <div class="child header-child title title1 simage">"StatesMC"</div>
            <div class="child header-child title title2" id="ip-parent">
                "Best Gun Server in the World!!!"
                <div class="infant">"IP: statesmc.us"</div>
            </div>
        </div>
    }
}

#[component]
fn Body() -> impl IntoView {
    view! {
        <div class="body parent">
            <Description />
            <elements::videoselector::VideoSection />
        </div>
    }
}

#[component]
fn Description() -> impl IntoView {
    view! {
        <div class="child body-child body-side">
            "Why don't civilization servers ever feature firearms? StatesMC makes this vision a reality. Unlike other (unmodded) SMPs, you can craft guns, technology, and nations capable of dominating continents. With the tools we give you, anything is possible. Lead a ruthless dictatorship set on conquering lands, or a thriving democracy serving the interests of its people. The choice is yours, and choose wisely. When chaos inevitably erupts, brace yourself for intense full scale gunfights that test the true capabilities of your nation. In the end, only the strongest factions may survive. The question is:
            Will it be yours?"
        </div>
    }
}
#[component]
fn MapSection() -> impl IntoView {
    view! {
        <div class="parent map-parent">
            <iframe
                style="width: 100%; height: 100%; overflow: hidden;"
                src="https://map.statesmc.us"
                width="400px"
                height="400px"
                scrolling="no"
                class="map-child"
            >
                "Failed to load the server map"
            </iframe>
            <LinkSection />
        </div>
    }
}

#[component]
fn LinkSection() -> impl IntoView {
    let links = link_choices();
    let audio_ref = NodeRef::<leptos::html::Audio>::new();
    let play_sound = move |_| {
        if let Some(audio) = audio_ref.get() {
            audio.set_current_time(0.0);
            let _ = audio.play();
        }
    };
    view! {
        <div>
            <audio node_ref=audio_ref src="/assets/magic.mp3" preload="auto"></audio>
            <div>
                {links
                    .into_iter()
                    .map(|link| {
                        view! {
                            <button class="text butt title title1 space" on:click=play_sound>
                                <A href=link.href target="_blank">
                                    {link.label}
                                </A>
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}
struct Link {
    label: &'static str,
    href: &'static str,
}
macro_rules! plinsh {
    ($name:expr, $url:expr, $vec:expr) => {
        $vec.push(Link {
            label: $name,
            href: $url,
        })
    };
}
fn link_choices() -> Vec<Link> {
    let mut l = vec![];
    plinsh!("About", "/about", l);
    plinsh!("Adventure", "/adventure", l);
    plinsh!("Games", "/gamepage", l);
    plinsh!("Discord", "https://discord.gg/RpAKqaz7by", l);
    plinsh!("Youtube", "https://www.youtube.com/@burvy", l);
    plinsh!("burvy.dev", "https://burvy.dev", l);
    l
}
