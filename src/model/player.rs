use serde::{Deserialize, Serialize};

use super::{boolean::Boolean, player_state::PlayerStates, vector2f::Vector2f};

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Player {
    position: Vector2f,
    position_counter: Vector2f,
    wall_stick_max: f32,
    speed: Vector2f,
    state: PlayerStates,
    //facing: Facing,
    aiming: Boolean,
    index: i32,
}
