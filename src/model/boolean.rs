use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum Boolean {
    IsOn = 1,
    IsOff = 0,
}

impl Default for Boolean {
    fn default() -> Self {
        Boolean::IsOff
    }
}

impl Boolean {
    pub fn is_on(&self) -> bool {
        self == &Boolean::IsOn
    }

    pub fn is_off(&self) -> bool {
        self == &Boolean::IsOff
    }
}
