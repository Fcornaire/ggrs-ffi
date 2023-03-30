use serde::{Deserialize, Serialize};

use crate::model::bool_ffi::BoolFFI;

use super::miasma_ffi::MiasmaFFI;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize, Default)]
pub struct SessionFFI {
    pub round_end_counter: f32,
    pub is_ending: BoolFFI,
    pub miasma: MiasmaFFI,
}
