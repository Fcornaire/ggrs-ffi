use serde::{Deserialize, Serialize};

use super::{arrow_types::ArrowTypes, vector2f::Vector2f};

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Arrow {
    pub position: Vector2f,
    pub arrow_type: ArrowTypes,
    pub player_index: i32,
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

    pub fn arrow_type(&self) -> ArrowTypes {
        self.arrow_type
    }

    pub fn player_index(&self) -> i32 {
        self.player_index
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

    pub fn arrow_type(mut self, arrow_type: ArrowTypes) -> ArrowBuilder {
        self.arrow.arrow_type = arrow_type;
        self
    }

    pub fn player_index(mut self, player_index: i32) -> ArrowBuilder {
        self.arrow.player_index = player_index;
        self
    }

    pub fn build(self) -> Arrow {
        self.arrow
    }
}
