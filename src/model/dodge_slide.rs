use serde::{Deserialize, Serialize};

use super::bool_ffi::BoolFFI;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize, Default)]
pub struct DodgeSlide {
    pub is_dodge_sliding: BoolFFI,
    pub was_dodge_sliding: BoolFFI,
}

impl DodgeSlide {
    pub fn new(is_dodge_sliding: BoolFFI, was_dodge_sliding: BoolFFI) -> Self {
        Self {
            is_dodge_sliding,
            was_dodge_sliding,
        }
    }

    pub fn is_dodge_sliding(&self) -> BoolFFI {
        self.is_dodge_sliding.clone()
    }

    pub fn was_dodge_sliding(&self) -> BoolFFI {
        self.was_dodge_sliding.clone()
    }
}
