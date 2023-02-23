use serde::{Deserialize, Serialize};

use crate::model::bool_ffi::BoolFFI;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct DodgeSlideFFI {
    pub is_dodge_sliding: BoolFFI,
    pub was_dodge_sliding: BoolFFI,
}
