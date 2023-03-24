use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum PlayerDraw {
    Player1 = 0,
    Player2 = 1,
    Unkown = 2,
}
