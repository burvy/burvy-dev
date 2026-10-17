use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::mainpage;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <p>"Page not found."</p> }>
            <Route path=path!("/about") view=About />
            <Route path=path!("/") view=mainpage::HomePage />
        </Routes>
    }
}

#[component]
fn About() -> impl IntoView {
    let (image, set_image) = signal(image_selector(1 as usize));
    let butts = button_generator();
    view! {
        <div class="header parent">
            <div class="title title1 simage header-child">"About StatesMC"</div>
            <div class="header-child">

                <button class="title title1 butt space">
                    <A href="/">"Home"</A>
                </button>
            </div>
        </div>
        <div class="body-parent">
            <p class="text">
                "
                Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
                "
            </p>
            <div
                class="limage"
                style:background-image=move || format!("url({})", image.get().image)
            >
                <div class="text">{move || image.get().message}</div>
                <div class="scroll-container">
                    {butts
                        .into_iter()
                        .map(|b| {
                            view! {
                                <li>
                                    <button
                                        class="butt-list"
                                        on:click=move |_| {
                                            set_image.set(image_selector(b.id as usize))
                                        }
                                    >
                                        {b.label}
                                    </button>
                                </li>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </div>
        <div>
            <A href="https://github.com/Burvye/statesmc-leptos-site/tree/main">
                <button class="github-link link-box"></button>
            </A>
            <A href="https://discord.gg/RpAKqaz7by">
                <button class="discord-link link-box"></button>
            </A>
        </div>
    }
}

#[derive(Clone)]
struct Button {
    id: i32,
    label: String,
}

fn button_generator() -> Vec<Button> {
    let mut buttons = vec![];
    for i in 1..=20 {
        buttons.push(Button {
            id: i,
            label: format!("Image {}", i),
        })
    }
    buttons
}

#[derive(Clone)]
struct Image {
    image: String,
    message: String,
}

impl Image {
    fn new(id: usize, description: String) -> Image {
        Image {
            image: format!("assets/backs/{}.png", id),
            message: description,
        }
    }
}

fn image_selector(id: usize) -> Image {
    const DESCRIPTIONS: &[&str] = &["Estglory dominating the hardpoint", "Estglory in battle"];
    image_selector_output(DESCRIPTIONS, id)
}
fn image_selector_output(descs: &[&str], id: usize) -> Image {
    id.checked_sub(1).and_then(|i| descs.get(i)).map_or_else(
        || Image {
            image: format!("67"),
            message: String::from("end it all"),
        },
        |desc| Image::new(id, desc.to_string()),
    )
}
