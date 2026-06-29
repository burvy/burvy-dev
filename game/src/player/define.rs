use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::prelude::*;

#[derive(Component, Clone, Default)]
pub struct PlayerData {
    pos: Vec3,
}

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
pub enum PlayerScheme {
    Jump(TnuaBuiltinJump),
}

pub fn build_scene(mut cmds: Commands) {
    cmds.spawn((
        RigidBody::Dynamic,
        Collider::capsule(0.5, 1.0),
        spawn_player(),
    ));
}

fn spawn_player() -> impl Scene {
    bsn! {
        Transform::from_xyz(0.0, 5.0, 0.0)
        PlayerData::default()
    }
}
