use bevy::prelude::*;
use bevy_tnua::prelude::*;

use super::define::WantMove;

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
