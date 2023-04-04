use serde::{Deserialize, Serialize};

use super::bool_ffi::BoolFFI;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize, Default)]
pub struct Flash {
    pub is_flashing: BoolFFI,
    pub flash_counter: f32,
}

impl Flash {
    pub fn new(is_flashing: BoolFFI, flash_counter: f32) -> Self {
        Self {
            is_flashing,
            flash_counter,
        }
    }

    pub fn is_flashing(&self) -> BoolFFI {
        self.is_flashing.clone()
    }

    pub fn flash_counter(&self) -> f32 {
        self.flash_counter.clone()
    }
}
