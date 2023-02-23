use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize, Eq, Default)]
pub enum BoolFFI {
    #[default]
    False = 0,
    True = 1,
}

unsafe impl Zeroable for BoolFFI {}
unsafe impl Pod for BoolFFI {}
