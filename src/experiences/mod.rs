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
        name: "Source Code",
        background: "images/github.png",
        description: "the source code for this website",
        path: "https://github.com/burvy/burvy-dev",
    },
    Experience {
        name: "StatesMC Website",
        description:
            "the website for my minecraft server - the only one with guns for java edition!",
        path: "https://statesmc.us/",
        background: "images/mwg.png",
    },
    Experience {
        name: "Chatroom",
        background: "images/chat.png",
        description: "chatroom chat",
        path: "/chat",
    },
    Experience {
        name: "Space Photos",
        background: "images/stars.png",
        description: "look at cool space photos from NASA!",
        path: "/space-photos",
    },
    Experience {
        name: "Soundboard",
        background: "images/sound.png",
        description: "play some of the sounds i have on this website!",
        path: "/soundboard",
    },
    Experience {
        name: "Cellular Automata",
        background: "images/life.png",
        description: "funny game of life",
        path: "/life",
    },
    Experience {
        name: "Game",
        background: "images/game.png",
        description: "game made with bevy on the web",
        path: "/game",
    },
];

/// also add the pages here as you add experiences
pub mod chat;
pub mod game;
pub mod life;
pub mod soundboard;
pub mod space;
pub mod test;
