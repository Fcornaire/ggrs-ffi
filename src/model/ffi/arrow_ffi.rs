use core::slice;

use crate::{
    model::{
        arrow::Arrow, arrow_states::ArrowStates, arrow_types::ArrowTypes, bool_ffi::BoolFFI,
        vector2f::Vector2f,
    },
    utils::{byte_array_to_guid, string_guid_to_byte_array},
};

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ArrowFFI {
    pub position: Vector2f,
    pub position_counter: Vector2f,
    pub speed: Vector2f,
    pub direction: f32,
    pub shooting_counter: f32,
    pub cannot_pickup_counter: f32,
    pub cannot_catch_counter: f32,
    pub state: ArrowStates,
    pub arrow_type: ArrowTypes,
    pub stuck_direction: Vector2f,
    pub player_index: i32,
    pub id: *mut u8,
    pub is_collidable: BoolFFI,
    pub is_active: BoolFFI,
    pub is_frozen: BoolFFI,
}

impl ArrowFFI {
    pub unsafe fn to_model(&self) -> Arrow {
        let guid = byte_array_to_guid(self.id);

        Arrow::builder()
            .position(self.position)
            .position_counter(self.position_counter)
            .direction(self.direction)
            .speed(self.speed)
            .shooting_counter(self.shooting_counter)
            .cannot_pickup_counter(self.cannot_pickup_counter)
            .cannot_catch_counter(self.cannot_catch_counter)
            .state(self.state)
            .arrow_type(self.arrow_type)
            .stuck_direction(self.stuck_direction)
            .player_index(self.player_index)
            .id(guid.to_string())
            .is_active(self.is_active)
            .is_collidable(self.is_collidable)
            .is_frozen(self.is_frozen)
            .build()
    }

    pub unsafe fn update(&mut self, arrow: Arrow) {
        self.position = arrow.position();
        self.position_counter = arrow.position_counter();
        self.direction = arrow.direction();
        self.speed = arrow.speed();
        self.shooting_counter = arrow.shooting_counter();
        self.cannot_pickup_counter = arrow.cannot_pickup_counter();
        self.cannot_catch_counter = arrow.cannot_catch_counter();
        self.state = arrow.state();
        self.arrow_type = arrow.arrow_type();
        self.stuck_direction = arrow.stuck_direction();
        self.player_index = arrow.player_index();
        self.is_active = arrow.is_active();
        self.is_collidable = arrow.is_collidable();
        self.is_frozen = arrow.is_frozen();

        let bytes = string_guid_to_byte_array(arrow.id());
        slice::from_raw_parts_mut(self.id, 16).copy_from_slice(&bytes);
    }

    pub fn is_empty_arrow(&self) -> bool {
        self.position.x == -1.0 && self.position.y == -1.0
    }
}
