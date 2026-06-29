mod player;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::TnuaControllerPlugin;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;

fn main() -> AppExit {
    App::new().add_plugins((DefaultPlugins, MainPlugin)).run()
}

pub struct MainPlugin;
impl Plugin for MainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PhysicsPlugins::default(),
            TnuaControllerPlugin::<player::define::PlayerScheme>::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
        ));
        app.add_plugins(player::PlayerPlugin);
    }
}
