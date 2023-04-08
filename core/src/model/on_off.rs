use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum OnOff {
    IsOn = 1,
    IsOff = 0,
}

impl Default for OnOff {
    fn default() -> Self {
        OnOff::IsOff
    }
}

impl OnOff {
    pub fn is_on(&self) -> bool {
        self == &OnOff::IsOn
    }

    pub fn is_off(&self) -> bool {
        self == &OnOff::IsOff
    }
}

unsafe impl Zeroable for OnOff {}
unsafe impl Pod for OnOff {}
