use crate::model::{arrow::Arrow, arrow_types::ArrowTypes, vector2f::Vector2f};

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ArrowFFI {
    pub position: Vector2f,
    pub arrow_type: ArrowTypes,
    pub player_index: i32,
}

impl ArrowFFI {
    pub unsafe fn to_model(&self) -> Arrow {
        Arrow::builder()
            .position(self.position)
            .arrow_type(self.arrow_type)
            .player_index(self.player_index)
            .build()
    }

    pub fn update(&mut self, arrow: Arrow) {
        self.position = arrow.position();
        self.arrow_type = arrow.arrow_type();
        self.player_index = arrow.player_index();
    }
}
