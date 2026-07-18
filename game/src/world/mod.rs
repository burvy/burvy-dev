use bevy::prelude::*;

pub mod definition;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, definition::setup_world.spawn());
    }
}
