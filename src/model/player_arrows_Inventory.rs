use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct PlayerArrowsInventory {
    pub normal: i32,
}
