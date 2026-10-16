/// a clickable experience! its like roblox they call their games "experiences" its so dumb
/// dont touch this btw
#[derive(Clone, Copy)]
pub struct Experience {
    pub name: &'static str,
    pub description: &'static str,
    // this must be set in `app.rs`
    pub path: &'static str,
    // the image card size is h: 1, w: 2, paint is good, 2000x1000
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
        description: "the website for my minecraft server - the only one with guns for java edition!",
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
        description: "DISCONTINUED (old web game)",
        path: "/game",
    },
    Experience {
        name: "Planner",
        background: "images/planner.png",
        description: "a planner app on the web",
        path: "/planner",
    },
    Experience {
        name: "Floret",
        background: "images/floret.png",
        description: "a little hangout space! walk around and talk to whoever is on",
        path: "/floret",
    },
    Experience {
        name: "Godot Game",
        background: "images/3d-learn.png",
        description: "learning 3D in godot",
        path: "/3d-learn",
    },
    Experience {
        name: "Venture",
        background: "images/venture.png",
        description: "a singleplayer battle simulator!",
        path: "/venture",
    },
    Experience {
        name: "Halcyon",
        background: "images/halcyon.png",
        description: "a real multiplayer game iterated quickly in godot",
        path: "/halcyon",
    },
    Experience {
        name: "Amity",
        background: "images/amity.png",
        description: "an FPS game based around open world exploration in a voxel enviroment",
        path: "/amity",
    },
    Experience {
        name: "Primer",
        // placeholder image, to be drawn (2000x1000)
        background: "images/primer.png",
        description: "a picture-book way to learn maths, read to you by Ada",
        path: "/primer",
    },
    Experience {
        name: "Rambl",
        background: "images/rambl.png",
        description: "ramble your day at it, and it writes your to-do list and plans your week (made with Jev at c0mpiled-15)",
        path: "https://rambl.burvy.dev",
    },
    Experience {
        name: "PSU PL Group",
        background: "images/plgroup.png",
        description: "Website for the Penn State Programming Languages Group",
        path: "/plgroup",
    },
    Experience {
        name: "Buy Me a Coffee",
        // placeholder image, to be drawn (2000x1000)
        background: "images/coffee.png",
        description: "if you like what i make, you can buy me a coffee!",
        path: "https://buymeacoffee.com/burvy",
    },
];

/// also add the pages here as you add experiences
pub mod amity;
pub mod chat;
pub mod floret;
pub mod game;
pub mod halcyon;
pub mod learn3d;
pub mod life;
pub mod planner;
pub mod primer;
pub mod soundboard;
pub mod space;
pub mod test;
pub mod venture;
