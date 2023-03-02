use crate::model::{
    arrow::Arrow, arrow_states::ArrowStates, arrow_types::ArrowTypes, vector2f::Vector2f,
};

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ArrowFFI {
    pub position: Vector2f,
    pub position_counter: Vector2f,
    pub speed: Vector2f,
    pub direction: f32,
    pub shooting_counter: f32,
    pub state: ArrowStates,
    pub arrow_type: ArrowTypes,
    pub stuck_direction: Vector2f,
    pub player_index: i32,
    pub index: i32,
}

impl ArrowFFI {
    pub unsafe fn to_model(&self) -> Arrow {
        Arrow::builder()
            .position(self.position)
            .position_counter(self.position_counter)
            .direction(self.direction)
            .speed(self.speed)
            .shooting_counter(self.shooting_counter)
            .state(self.state)
            .arrow_type(self.arrow_type)
            .stuck_direction(self.stuck_direction)
            .player_index(self.player_index)
            .index(self.index)
            .build()
    }

    pub fn update(&mut self, arrow: Arrow) {
        self.position = arrow.position();
        self.position_counter = arrow.position_counter();
        self.direction = arrow.direction();
        self.speed = arrow.speed();
        self.shooting_counter = arrow.shooting_counter();
        self.state = arrow.state();
        self.arrow_type = arrow.arrow_type();
        self.stuck_direction = arrow.stuck_direction();
        self.player_index = arrow.player_index();
        self.index = arrow.index();
    }
}
