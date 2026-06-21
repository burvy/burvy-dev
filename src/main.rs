mod app;
mod components;
mod experiences;
mod pages;

// wow so clean
fn main() {
    leptos::mount::mount_to_body(app::App);
}
