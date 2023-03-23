use serde::{Deserialize, Serialize};

use crate::model::bool_ffi::BoolFFI;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize, Default)]
pub struct SessionFFI {
    pub round_end_counter: f32,
    pub is_ending: BoolFFI,
}
