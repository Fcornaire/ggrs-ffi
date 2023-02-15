use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use super::{boolean::Boolean, vector2f::Vector2f};

unsafe impl Zeroable for Boolean {}
unsafe impl Pod for Boolean {}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable, Default)]
pub struct Input {
    jump_check: Boolean,
    jump_pressed: Boolean,
    shoot_check: Boolean,
    shoot_pressed: Boolean,
    alt_shoot_check: Boolean,
    alt_shoot_pressed: Boolean,
    dodge_check: Boolean,
    dodge_pressed: Boolean,
    arrow_pressed: Boolean,
    move_x: i32,
    move_y: i32,
    aim_axis: Vector2f,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Default, Deserialize, PartialOrd, PartialEq)]
pub struct PressedButton {
    pub jump: Boolean,
    pub shield: Boolean,
}
