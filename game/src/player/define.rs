use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::{
    builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig},
    prelude::*,
};

#[derive(Component, Clone, Default)]
pub struct PlayerData {
    pos: Vec3, // TODO: update pos as well so we can send it over the network later on
}

#[derive(Component, Clone, Default)] // compiler happi
pub struct MainPlayer;

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
pub enum PlayerScheme {
    Jump(TnuaBuiltinJump),
}

/// thing to actually spawn the entity
pub fn build_scene(mut cmds: Commands, mut configs: ResMut<Assets<PlayerSchemeConfig>>) {
    let config = configs.add(PlayerSchemeConfig {
        basis: TnuaBuiltinWalkConfig {
            float_height: 2.0,
            ..default()
        },
        jump: TnuaBuiltinJumpConfig {
            height: super::input::JUMP_VEL,
            ..default()
        },
    });
    let entity = cmds.spawn_scene(spawn_player()).id();
    // add additional things here the legacy way
    cmds.entity(entity).insert((
        RigidBody::Dynamic,
        Collider::capsule(0.5, 1.0),
        TnuaController::<PlayerScheme>::default(),
        TnuaConfig::<PlayerScheme>(config),
    ));
}
/// put bsn macro stuff in here and unfortunately it is not fully supported
/// so stuff like avian 3d stuff and wahtever put in the build scene additional inserts
fn spawn_player() -> impl Scene {
    bsn! {
        Transform::from_xyz(0.0, 5.0, 0.0)
        PlayerData
        MainPlayer // yes we are the main player
        Children [
            (
                Camera3d
            )
        ]
    }
}
