use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq, Default)]
pub enum ArrowStates {
    #[default]
    Shooting = 0,
    Drilling = 1,
    Gravity = 2,
    Falling = 3,
    Stuck = 4,
    LayingOnGround = 5,
    Buried = 6,
}
