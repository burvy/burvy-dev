use bevy::prelude::*;
use bevy_tnua::prelude::*;

use super::define::WantMove;

const SPEED: f32 = 6.7;
pub const JUMP_VEL: f32 = 17.6867;

pub fn player_move(
    mut player: Single<
        (
            &mut TnuaController<super::define::PlayerScheme>,
            &mut super::define::PlayerData,
        ),
        With<super::define::MainPlayer>,
    >,
) {
}
