use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use super::vector2f::Vector2f;

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum State {
    IsOn = 1,
    IsOff = 0,
}

impl Default for State {
    fn default() -> Self {
        State::IsOff
    }
}

impl State {
    pub fn is_on(&self) -> bool {
        self == &State::IsOn
    }

    pub fn is_off(&self) -> bool {
        self == &State::IsOff
    }
}

unsafe impl Zeroable for State {}
unsafe impl Pod for State {}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable, Default)]
pub struct Input {
    jump_check: State,
    jump_pressed: State,
    shoot_check: State,
    shoot_pressed: State,
    alt_shoot_check: State,
    alt_shoot_pressed: State,
    dodge_check: State,
    dodge_pressed: State,
    arrow_pressed: State,
    move_x: i32,
    move_y: i32,
    aim_axis: Vector2f,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Default, Deserialize, PartialOrd, PartialEq)]
pub struct PressedButton {
    pub jump: State,
    pub shield: State,
}
