use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq, Default)]
pub struct Vector2f {
    x: f32,
    y: f32,
}

unsafe impl Zeroable for Vector2f {}
unsafe impl Pod for Vector2f {}
