use bevy::prelude::*;
use bevy_tnua::prelude::*;

const SPEED: f32 = 10.0;

pub fn move_player(
    mut player: Single<
        (
            &mut TnuaController<super::define::PlayerScheme>,
            &mut super::define::PlayerData,
        ),
        With<super::define::MainPlayer>,
    >,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let mut desire: Vec3 = Vec3::ZERO;
    // TODO: replace with configurable keybinds later on
    if keys.pressed(KeyCode::KeyW) {
        desire.z += SPEED;
    }
    if keys.pressed(KeyCode::KeyS) {
        desire.z -= SPEED;
    }
    if keys.pressed(KeyCode::KeyA) {
        desire.x -= SPEED;
    }
    if keys.pressed(KeyCode::KeyD) {
        desire.x += SPEED;
    }
    player.basis = TnuaBuiltinWalk {
        desired_motion: desire,
        desired_forward: todo!(),
    }
}
