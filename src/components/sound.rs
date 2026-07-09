use leptos::prelude::*;

/// input the location of your sound file `/sounds/your_sound.mp3`
/// you can assign it to a variable like:
/// ```
/// var_sound = play_sound("/sounds/boom.mp3");
/// var_sound();
/// ```
/// or you can do:
/// ```
/// play_sound("/sounds/boom.mp3");
/// ```
/// forcing this to implement copy helps with borrow checker errors
pub fn play_sound(sound: &'static str) -> impl Fn() + Copy {
    move || {
        let _ = web_sys::HtmlAudioElement::new_with_src(sound).and_then(|audio| audio.play());
    }
}
