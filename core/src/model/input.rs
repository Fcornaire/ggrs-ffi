use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use super::vector2f::Vector2f;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Pod, Zeroable, Default, Serialize, Deserialize)]
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
}
