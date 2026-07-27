mod app;
mod components;
mod experiences;
mod lazy;
mod pages;
mod webtrans;

// wow so clean
fn main() {
    leptos::mount::mount_to_body(app::App);
}
