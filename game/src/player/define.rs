use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::prelude::*;

#[derive(Component, Clone, Default)]
pub struct PlayerData {
    /// the velocity a player wants to go
    /// should be reset every frame
    pos: Vec3,
}

#[derive(Component)]
pub struct MainPlayer;

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
pub enum PlayerScheme {
    Jump(TnuaBuiltinJump),
}

/// thing to actually spawn the entity
pub fn build_scene(mut cmds: Commands) {
    let entity = cmds.spawn_scene(spawn_player()).id();
    // add additional things here the legacy way
    cmds.entity(entity).insert((
        RigidBody::Dynamic,
        Collider::capsule(0.5, 1.0),
        TnuaController::<PlayerScheme>::default(),
        TnuaConfig::<PlayerScheme>(config_handle), // TODO: what here?
                                                   // playerdata handled in bsn macro
    ));
}
/// put bsn macro stuff in here and unfortunately it is not fully supported
/// so stuff like avian 3d stuff and wahtever put in the build scene additional inserts
fn spawn_player() -> impl Scene {
    bsn! {
        Transform::from_xyz(0.0, 5.0, 0.0)
        PlayerData::default()
    }
}
