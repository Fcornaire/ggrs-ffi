use serde::{Deserialize, Serialize};

use super::vector2f::Vector2f;

#[repr(C)]
#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    position: Vector2f,
    index: i32,
}
