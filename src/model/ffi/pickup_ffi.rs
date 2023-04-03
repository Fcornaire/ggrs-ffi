use core::slice;

use crate::{
    model::{bool_ffi::BoolFFI, pickup::Pickup, pickup_state::PickupState, vector2f::Vector2f},
    utils::{byte_array_to_guid, string_guid_to_byte_array},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct PickupFFI {
    pub state: PickupState,
    pub position: Vector2f,
    pub target_position: Vector2f,
    pub sine_counter: f32,
    pub target_position_timer: i32,
    pub is_collidable: BoolFFI,
    pub collidable_timer: i32,
    pub player_index: i32,
    pub id: *mut u8,
}

impl PickupFFI {
    pub unsafe fn to_model(&self) -> Pickup {
        let guid = byte_array_to_guid(self.id);

        Pickup {
            player_index: self.player_index,
            state: self.state,
            target_position: self.target_position,
            position: self.position,
            sine_counter: self.sine_counter,
            target_position_timer: self.target_position_timer,
            is_collidable: self.is_collidable,
            collidable_timer: self.collidable_timer,
            id: guid.to_string(),
        }
    }

    pub unsafe fn update(&mut self, pickup: Pickup) {
        self.player_index = pickup.player_index;
        self.position = pickup.position;
        self.target_position = pickup.target_position;
        self.sine_counter = pickup.sine_counter;
        self.target_position_timer = pickup.target_position_timer;
        self.collidable_timer = pickup.collidable_timer;
        self.is_collidable = pickup.is_collidable;
        self.state = pickup.state;

        let bytes = string_guid_to_byte_array(pickup.id);
        slice::from_raw_parts_mut(self.id, 16).copy_from_slice(&bytes);
    }

    pub fn is_empty(&self) -> bool {
        self.position.x == -1.0 && self.position.y == -1.0
    }
}
