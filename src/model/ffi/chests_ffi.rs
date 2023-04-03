use core::slice;

use crate::{
    model::{
        bool_ffi::BoolFFI, chest::Chest, chest_state::ChestState, pickup_state::PickupState,
        vector2f::Vector2f,
    },
    utils::{byte_array_to_guid, string_guid_to_byte_array},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct ChestFFI {
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
    pub id: *mut u8,
}

impl ChestFFI {
    pub unsafe fn to_model(&self) -> Chest {
        let guid = byte_array_to_guid(self.id);

        Chest {
            current_anim_id: self.current_anim_id,
            is_collidable: self.is_collidable,
            appear_counter: self.appear_counter,
            pickups: self.pickups,
            position: self.position,
            position_counter: self.position_counter,
            v_speed: self.v_speed,
            state: self.state,
            appear_timer: self.appear_timer,
            is_light_visible: self.is_light_visible,
            is_visible: self.is_visible,
            opening_timer: self.opening_timer,
            id: guid.to_string(),
        }
    }

    pub unsafe fn update(&mut self, chest: Chest) {
        self.current_anim_id = chest.current_anim_id;
        self.is_collidable = chest.is_collidable;
        self.position = chest.position;
        self.position_counter = chest.position_counter;
        self.v_speed = chest.v_speed;
        self.pickups = chest.pickups;
        self.appear_counter = chest.appear_counter;
        self.state = chest.state;
        self.is_light_visible = chest.is_light_visible;
        self.is_visible = chest.is_visible;
        self.appear_timer = chest.appear_timer;
        self.opening_timer = chest.opening_timer;

        let bytes = string_guid_to_byte_array(chest.id);
        slice::from_raw_parts_mut(self.id, 16).copy_from_slice(&bytes);
    }

    pub fn is_empty(&self) -> bool {
        self.position.x == -1.0 && self.position.y == -1.0
    }
}
