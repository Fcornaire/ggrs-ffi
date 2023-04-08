use bytemuck::{Pod, Zeroable};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{on_off::OnOff, vector2f::Vector2f};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Pod, Zeroable, Default, Serialize, Deserialize)]
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

    pub fn jump() -> Self {
        Self {
            jump_check: OnOff::IsOn,
            jump_pressed: OnOff::IsOn,
            shoot_check: OnOff::IsOff,
            shoot_pressed: OnOff::IsOff,
            alt_shoot_check: OnOff::IsOff,
            alt_shoot_pressed: OnOff::IsOff,
            dodge_check: OnOff::IsOff,
            dodge_pressed: OnOff::IsOff,
            arrow_pressed: OnOff::IsOff,
            move_x: 0,
            move_y: 0,
            aim_axis: Vector2f::default(),
        }
    }

    pub fn dodge() -> Self {
        let mut rng = rand::thread_rng();

        let x: i32 = rng.gen_range(-2..2);
        let y: i32 = rng.gen_range(-2..2);

        Self {
            jump_check: OnOff::IsOff,
            jump_pressed: OnOff::IsOff,
            shoot_check: OnOff::IsOff,
            shoot_pressed: OnOff::IsOff,
            alt_shoot_check: OnOff::IsOff,
            alt_shoot_pressed: OnOff::IsOff,
            dodge_check: OnOff::IsOn,
            dodge_pressed: OnOff::IsOn,
            arrow_pressed: OnOff::IsOff,
            move_x: x,
            move_y: y,
            aim_axis: Vector2f {
                x: x as f32,
                y: y as f32,
            },
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Default, Deserialize, PartialOrd, PartialEq)]
pub struct PressedButton {
    pub jump: OnOff,
    pub shield: OnOff,
}
