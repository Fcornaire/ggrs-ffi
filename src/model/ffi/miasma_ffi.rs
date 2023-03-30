use serde::{Deserialize, Serialize};

use crate::model::bool_ffi::BoolFFI;

use super::miasma_state_ffi::MiasmaState;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize, Default)]
pub struct MiasmaFFI {
    pub counter: f32,
    pub state: MiasmaState,
    pub percent: f32,
    pub coroutine_timer: i32,
    pub is_collidable: BoolFFI,
}
