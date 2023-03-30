use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq, Default)]
pub enum MiasmaState {
    #[default]
    Uninitialized = 0,
    Initialized = 1,
    Collidable = 2,
    Phase2 = 3,
    NervesOfSteel = 4,
    End = 5,
}
