use serde::{Deserialize, Serialize};

use super::{bool_ffi::BoolFFI, pickup_state::PickupState, vector2f::Vector2f};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Pickup {
    pub state: PickupState,
    pub position: Vector2f,
    pub target_position: Vector2f,
    pub sine_counter: f32,
    pub target_position_timer: i32,
    pub is_collidable: BoolFFI,
    pub collidable_timer: i32,
    pub player_index: i32,
    pub id: String,
}
