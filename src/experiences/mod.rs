/// a clickable experience! its like roblox they call their games "experiences" its so dumb
/// dont touch this btw
#[derive(Clone, Copy)]
pub struct Experience {
    pub name: &'static str,
    pub description: &'static str,
    // this must be set in `app.rs`
    pub path: &'static str,
    // the image card size is h: 1, w: 4, paint is good, 2000x500
    pub background: &'static str,
}

/// add new experiences here
/// theyre just structs
pub const EXPERIENCES: &[Experience] = &[
    Experience {
        name: "Test",
        background: "images/glitchcity.jpg",
        description: "test experience to make sure everything works",
        path: "/test",
    },
    Experience {
        name: "Chatroom",
        background: "images/chat.png",
        description: "chatroom chat",
        path: "/chat",
    },
];

pub mod chat;
/// also add the pages here as you add experiences
pub mod test;
