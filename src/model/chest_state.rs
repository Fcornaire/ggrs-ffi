use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq, Default)]
pub enum ChestState {
    #[default]
    WaitingToAppear = 0,
    Appearing = 1,
    Closed = 2,
    Opening = 3,
    Opened = 4,
}
