use std::ops::DerefMut;

use bevy::prelude::*;
use bevy_tnua::prelude::*;

use super::define::{PlayerData, WantMove};

const SPEED: f32 = 5.0;
pub const JUMP_VEL: f32 = 10.0;

pub fn player_input(
    mut player: Single<
        (
            &mut TnuaController<super::define::PlayerScheme>,
            &mut PlayerData,
            &Transform,
        ),
        With<super::define::MainPlayer>,
    >,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // TODO: use the other of these playerdata and transform
    let (tcon, pdata, tf) = player.deref_mut(); // remember this lol
    let desire = WantMove {
        x: keys.pressed(KeyCode::KeyD) as i8 - keys.pressed(KeyCode::KeyA) as i8,
        z: keys.pressed(KeyCode::KeyW) as i8 - keys.pressed(KeyCode::KeyS) as i8,
        jump: false,
    };
    let dir = Vec3 {
        x: desire.x as f32 * SPEED,
        y: 0.0,
        z: desire.z as f32 * SPEED,
    };

    tcon.basis = TnuaBuiltinWalk {
        desired_motion: dir,
        desired_forward: Some(Dir3::Z), // WHAT THE HECK WHY WHY SOME WHY
    };
    if keys.pressed(KeyCode::Space) {
        tcon.action(super::define::PlayerScheme::Jump(TnuaBuiltinJump {
            ..default() // sorryy no height field here
        }));
    }
}
