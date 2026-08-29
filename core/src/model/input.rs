use super::vector2f::Vector2f;
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Input {
    jump_check: i32,
    jump_pressed: i32,
    shoot_check: i32,
    shoot_pressed: i32,
    alt_shoot_check: i32,
    alt_shoot_pressed: i32,
    dodge_check: i32,
    dodge_pressed: i32,
    arrow_pressed: i32,
    move_x: i32,
    move_y: i32,
    aim_axis: Vector2f,
    aim_right_axis: Vector2f,
    disconnected: i32,
}

impl Input {
    pub fn disconnected() -> Self {
        Self {
            disconnected: 1,
            ..Default::default()
        }
    }
}
