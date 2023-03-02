use serde::{Deserialize, Serialize};

use super::{arrow_states::ArrowStates, arrow_types::ArrowTypes, vector2f::Vector2f};

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Arrow {
    position: Vector2f,
    position_counter: Vector2f,
    direction: f32,
    speed: Vector2f,
    shooting_counter: f32,
    state: ArrowStates,
    arrow_type: ArrowTypes,
    stuck_direction: Vector2f,
    player_index: i32,
    index: i32,
}

impl Arrow {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn builder() -> ArrowBuilder {
        ArrowBuilder::default()
    }

    pub fn position(&self) -> Vector2f {
        self.position
    }

    pub fn position_counter(&self) -> Vector2f {
        self.position_counter
    }

    pub fn direction(&self) -> f32 {
        self.direction
    }

    pub fn shooting_counter(&self) -> f32 {
        self.shooting_counter
    }

    pub fn speed(&self) -> Vector2f {
        self.speed
    }

    pub fn state(&self) -> ArrowStates {
        self.state
    }

    pub fn arrow_type(&self) -> ArrowTypes {
        self.arrow_type
    }

    pub fn stuck_direction(&self) -> Vector2f {
        self.stuck_direction
    }

    pub fn player_index(&self) -> i32 {
        self.player_index
    }

    pub fn index(&self) -> i32 {
        self.index
    }
}

#[derive(Default)]
pub struct ArrowBuilder {
    arrow: Arrow,
}

impl ArrowBuilder {
    pub fn new() -> ArrowBuilder {
        ArrowBuilder {
            arrow: Arrow::new(),
        }
    }

    pub fn position(mut self, position: Vector2f) -> ArrowBuilder {
        self.arrow.position = position;
        self
    }

    pub fn position_counter(mut self, position_counter: Vector2f) -> ArrowBuilder {
        self.arrow.position_counter = position_counter;
        self
    }

    pub fn direction(mut self, direction: f32) -> ArrowBuilder {
        self.arrow.direction = direction;
        self
    }

    pub fn speed(mut self, speed: Vector2f) -> ArrowBuilder {
        self.arrow.speed = speed;
        self
    }

    pub fn shooting_counter(mut self, shooting_counter: f32) -> ArrowBuilder {
        self.arrow.shooting_counter = shooting_counter;
        self
    }

    pub fn state(mut self, state: ArrowStates) -> ArrowBuilder {
        self.arrow.state = state;
        self
    }

    pub fn arrow_type(mut self, arrow_type: ArrowTypes) -> ArrowBuilder {
        self.arrow.arrow_type = arrow_type;
        self
    }

    pub fn stuck_direction(mut self, stuck_direction: Vector2f) -> ArrowBuilder {
        self.arrow.stuck_direction = stuck_direction;
        self
    }

    pub fn player_index(mut self, player_index: i32) -> ArrowBuilder {
        self.arrow.player_index = player_index;
        self
    }

    pub fn index(mut self, index: i32) -> ArrowBuilder {
        self.arrow.index = index;
        self
    }

    pub fn build(self) -> Arrow {
        self.arrow
    }
}
