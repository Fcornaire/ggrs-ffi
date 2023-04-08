use std::mem::forget;

use serde::{Deserialize, Serialize};

use super::safe_bytes::SafeBytes;

#[repr(C)]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UnmanagedBytes {
    bytes: Vec<u8>,
    pub size: usize,
}

impl UnmanagedBytes {
    pub fn empty() -> Self {
        Self {
            bytes: vec![],
            size: 0,
        }
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    pub fn new(safe_bytes: SafeBytes) -> Self {
        let byte_vec = unsafe { safe_bytes.slice() };
        let vec = byte_vec.to_vec();
        let size = vec.len();

        Self {
            bytes: vec.clone(),
            size,
        }
    }

    pub fn to_safe_bytes(&mut self) -> SafeBytes {
        let mut data = self.bytes.clone();

        let safe = Box::new(SafeBytes::new(data.as_mut_ptr(), self.size));

        forget(data);

        *safe
    }
}
