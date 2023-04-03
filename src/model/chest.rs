use serde::{Deserialize, Serialize};

use super::{
    bool_ffi::BoolFFI, chest_state::ChestState, pickup_state::PickupState, vector2f::Vector2f,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Chest {
    pub current_anim_id: i32,
    pub is_collidable: BoolFFI,
    pub appear_counter: f32,
    pub pickups: PickupState,
    pub position: Vector2f,
    pub position_counter: Vector2f,
    pub v_speed: f32,
    pub state: ChestState,
    pub appear_timer: i32,
    pub is_visible: BoolFFI,
    pub is_light_visible: BoolFFI,
    pub opening_timer: i32,
    pub id: String,
}
