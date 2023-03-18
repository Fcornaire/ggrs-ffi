use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use super::{on_off::OnOff, vector2f::Vector2f};

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable, Default)]
pub struct Input {
    jump_check: OnOff,
    jump_pressed: OnOff,
    shoot_check: OnOff,
    shoot_pressed: OnOff,
    alt_shoot_check: OnOff,
    alt_shoot_pressed: OnOff,
    dodge_check: OnOff,
    dodge_pressed: OnOff,
    arrow_pressed: OnOff,
    move_x: i32,
    move_y: i32,
    aim_axis: Vector2f,
}

impl Input {
    pub fn arrow_pressed() -> Self {
        Self {
            jump_check: OnOff::IsOff,
            jump_pressed: OnOff::IsOff,
            shoot_check: OnOff::IsOn,
            shoot_pressed: OnOff::IsOn,
            alt_shoot_check: OnOff::IsOff,
            alt_shoot_pressed: OnOff::IsOff,
            dodge_check: OnOff::IsOff,
            dodge_pressed: OnOff::IsOff,
            arrow_pressed: OnOff::IsOn,
            move_x: 0,
            move_y: 0,
            aim_axis: Vector2f::default(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Default, Deserialize, PartialOrd, PartialEq)]
pub struct PressedButton {
    pub jump: OnOff,
    pub shield: OnOff,
}
