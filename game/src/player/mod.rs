pub mod define;
pub mod input;
pub mod logic;

use bevy::prelude::*;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, define::build_scene);
        app.add_systems(Update, input::player_input);
    }
}
