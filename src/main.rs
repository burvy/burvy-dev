mod app;
mod components;
mod experiences;
mod pages;

fn main() {
    leptos::mount::mount_to_body(app::App);
}
