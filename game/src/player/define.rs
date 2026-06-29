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

fn spawn_player(cmds: &mut Commands) {
    cmds.spawn(bsn! {
        Player {
            Transform::from_xyz(0.0, 5.0, 0.0),
            RigidBody::Dynamic,
            Collider::capsule(0.5, 1.0),
            PlayerData::default(),
        }
    });
}

fn spawn_player_legacy(cmds: &mut Commands) {
    cmds.spawn((
        Transform::from_xyz(0.0, 5.0, 0.0),
        RigidBody::Dynamic,
        Collider::capsule(0.5, 1.0),
        PlayerData::default(),
    ));
}
