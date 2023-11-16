use super::vector2f::Vector2f;
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Pod, Zeroable, Default, Serialize, Deserialize)]
pub struct Input {
    jump_check: usize,
    jump_pressed: usize,
    shoot_check: usize,
    shoot_pressed: usize,
    alt_shoot_check: usize,
    alt_shoot_pressed: usize,
    dodge_check: usize,
    dodge_pressed: usize,
    arrow_pressed: usize,
    move_x: usize,
    move_y: usize,
    aim_axis: Vector2f,
    aim_right_axis: Vector2f,
}
