use serde::{Deserialize, Serialize};

use super::{boolean::Boolean, player_state::PlayerStates, vector2f::Vector2f};

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Player {
    position: Vector2f,
    speed: Vector2f,
    state: PlayerStates,
    //facing: Facing,
    aiming: Boolean,
    invisible: Boolean,
    invis_opacity: f32,
    index: i32,
}
