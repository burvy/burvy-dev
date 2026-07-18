use bevy::prelude::*;
use game::MainPlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MainPlugin)
        .run()
}
