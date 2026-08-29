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
        let bytes = unsafe { safe_bytes.slice() }.to_vec();
        let size = bytes.len();

        Self { bytes, size }
    }

    pub fn to_safe_bytes(&mut self) -> SafeBytes {
        let data = self.bytes.clone().into_boxed_slice();
        let size = data.len();

        SafeBytes::new(Box::into_raw(data) as *mut u8, size)
    }
}
