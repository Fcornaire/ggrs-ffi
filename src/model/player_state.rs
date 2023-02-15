use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum PlayerStates {
    Normal = 0,
    LedgeGrab = 1,
    Ducking = 2,
    Dodging = 3,
    Dying = 4,
    Frozen = 5,
}
