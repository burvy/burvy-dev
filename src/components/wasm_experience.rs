use leptos::prelude::*;
use wasm_bindgen::JsValue;

use crate::lazy;

/// A game made with bevy, loaded on demand into a full-page canvas, behind a
/// loading screen: its name, and a bar showing the download, until it's
/// running (see lazy.rs, which keeps the screen up to date and fades it out).
///
/// `script` is the game's wasm-bindgen module (e.g. "/amity/amity-wasm.js"),
/// and `canvas_id` the canvas id the game looks for.
#[component]
pub fn WasmExperience(
    title: &'static str,
    script: &'static str,
    canvas_id: &'static str,
) -> impl IntoView {
    let canvas = NodeRef::<leptos::html::Canvas>::new();
    let overlay = NodeRef::<leptos::html::Div>::new();

    // Wait for the canvas to exist before starting: bevy looks it up by id, and
    // starting first means it finds nothing and falls back to its own window.
    Effect::new(move |_| {
        if let (Some(_), Some(overlay)) = (canvas.get(), overlay.get()) {
            lazy::start_with_loading_screen(script, JsValue::from(overlay));
        }
    });

    view! {
        <div id="game-wrapper" style="position: relative;">
            <div node_ref=overlay class="loading-overlay">
                <div class="loading-title">{title}</div>
                <div class="loading-bar">
                    <div class="loading-bar-fill"></div>
                </div>
                <div class="loading-detail">"Loading…"</div>
            </div>
            <canvas node_ref=canvas id=canvas_id></canvas>
        </div>
    }
}
