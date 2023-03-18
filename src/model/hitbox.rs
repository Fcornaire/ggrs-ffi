use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq, Default)]
pub enum Hitbox {
    #[default]
    Normal = 0,
    Ducking = 1,
}
